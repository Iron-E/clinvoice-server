//! The `server` module functions to spawn an [`axum_server`] which communicates over TLS.

mod auth;
mod db_session_store;
mod handler;
mod response;
mod state;
#[cfg(test)]
mod test_client_ext;

use core::{fmt::Display, marker::PhantomData, time::Duration};
use std::net::SocketAddr;

use auth::{DbUserStore, InitializableWithAuthorization, RequireAuthLayer, UserStore};
use axum::{
	error_handling::HandleErrorLayer,
	http::{HeaderMap, Request, StatusCode},
	middleware::{self, Next},
	BoxError,
	Router,
};
use axum_login::{
	axum_sessions::{async_session::SessionStore, SessionLayer},
	AuthLayer,
	SqlxStore,
};
use axum_server::tls_rustls::RustlsConfig;
use db_session_store::DbSessionStore;
use handler::Handler;
pub use response::VersionResponse;
use semver::VersionReq;
use sqlx::{Connection, Database, Executor, QueryBuilder};
pub use state::ServerState;
use tower::{timeout, ServiceBuilder};
use tower_http::{compression::CompressionLayer, trace::TraceLayer};
use winvoice_adapter::{fmt::sql, Initializable};

use crate::{
	api::{self, routes},
	bool_ext::BoolExt,
	schema::{columns::UserColumns, Adapter, User},
	DynResult,
};

/// A Winvoice server.
#[derive(Clone, Debug)]
pub struct Server<A>
{
	/// The [`SocketAddr`] that self server is bound to.
	address: SocketAddr,

	phantom: PhantomData<A>,

	/// The TLS configuration.
	tls: RustlsConfig,
}

impl<A> Server<A>
where
	A: 'static + Adapter + InitializableWithAuthorization,
	<A::Db as Database>::Connection: core::fmt::Debug,
	<<A::Db as Database>::Connection as Connection>::Options: Clone,
	A::User: Default,
	DbSessionStore<A::Db>: Initializable<Db = A::Db> + SessionStore,
	DbUserStore<A::Db>: UserStore,
	for<'args> QueryBuilder<'args, A::Db>: From<A::User>,
	for<'connection> &'connection mut <A::Db as Database>::Connection: Executor<'connection, Database = A::Db>,
{
	/// Create a new [`Server`]
	pub const fn new(address: SocketAddr, tls: RustlsConfig) -> Self
	{
		Self { address, phantom: PhantomData, tls }
	}

	/// Create an [`Router`] based on the `connect_options`.
	///
	/// Operations `timeout`, if specified.
	pub async fn serve(
		self,
		cookie_domain: Option<String>,
		cookie_secret: Vec<u8>,
		state: ServerState<A::Db>,
		session_ttl: Duration,
		timeout: Option<Duration>,
	) -> DynResult<()>
	{
		let router = Self::router(cookie_domain, cookie_secret, state, session_ttl, timeout).await?;
		axum_server::bind_rustls(self.address, self.tls).serve(router.into_make_service()).await?;
		Ok(())
	}

	/// Create the [`Router`] that will be used by the [`Server`].
	async fn router(
		cookie_domain: Option<String>,
		cookie_secret: Vec<u8>,
		state: ServerState<A::Db>,
		session_ttl: Duration,
		timeout: Option<Duration>,
	) -> sqlx::Result<Router>
	{
		/// Middleware to check the [`api`] version of connecting clients.
		async fn version_checker<B>(
			headers: HeaderMap,
			req: Request<B>,
			next: Next<B>,
		) -> Result<axum::response::Response, VersionResponse>
		where
			B: core::fmt::Debug,
		{
			fn encoding_err<E>(e: E) -> Result<(), VersionResponse>
			where
				E: Display + ToString,
			{
				tracing::error!("Encoding error: {e}");
				Err(VersionResponse::encoding_err(e.to_string()))
			}

			let span = tracing::info_span!(
				"version_checker",
				headers = format!("{:?}", headers),
				req = format!("{:?}", req),
				next = format!("{:?}", next),
			);

			{
				let _ = span.enter();

				// do something with `request`...
				headers.get(api::HEADER).map_or_else(
					|| Err(VersionResponse::missing()),
					|version| {
						version.to_str().map_or_else(encoding_err, |v| {
							VersionReq::parse(v).map_or_else(encoding_err, |req| {
								req.matches(api::version()).then_some_or(Err(VersionResponse::mismatch()), Ok(()))
							})
						})
					},
				)?;
			}

			Ok(next.run(req).await)
		}

		let session_store = DbSessionStore::new(state.pool().clone());
		futures::try_join!(A::init_with_auth(state.pool()), session_store.init())?;

		let handler = Handler::<A>::new();
		let mut router = Router::new()
			.route(routes::CONTACT, handler.contact())
			.route(routes::DEPARTMENT, handler.department())
			.route(routes::EMPLOYEE, handler.employee())
			.route(routes::EXPENSE, handler.expense())
			.route(routes::JOB, handler.job())
			.route(routes::LOCATION, handler.location())
			.route(routes::LOGOUT, handler.logout())
			.route(routes::ORGANIZATION, handler.organization())
			.route(routes::ROLE, handler.role())
			.route(routes::TIMESHEET, handler.timesheet())
			.route(routes::USER, handler.user())
			.route_layer(RequireAuthLayer::login())
			.route(routes::LOGIN, handler.login());

		if let Some(t) = timeout
		{
			router = router.layer(
				ServiceBuilder::new()
					.layer(HandleErrorLayer::new(|err: BoxError| async move {
						err.is::<timeout::error::Elapsed>().then_or_else(
							|| (StatusCode::REQUEST_TIMEOUT, "Request took too long".to_owned()),
							|| (StatusCode::INTERNAL_SERVER_ERROR, format!("Unhandled internal error: {err}")),
						)
					}))
					.timeout(t),
			);
		}

		Ok(router
			.layer(AuthLayer::new(
				SqlxStore::<_, User>::new(state.pool().clone()).with_query({
					let mut query = QueryBuilder::<A::Db>::from(A::User::default());
					query.push(sql::WHERE).push(UserColumns::default().default_scope().id).push(" = $1");
					query.into_sql()
				}),
				&cookie_secret,
			))
			.layer({
				let mut layer = SessionLayer::new(session_store, &cookie_secret).with_session_ttl(session_ttl.into());

				if let Some(s) = cookie_domain
				{
					layer = layer.with_cookie_domain(s);
				}

				layer
			})
			.layer(middleware::from_fn(version_checker))
			.layer(CompressionLayer::new())
			.layer(TraceLayer::new_for_http())
			.with_state(state))
	}
}

#[allow(clippy::std_instead_of_core, clippy::str_to_string, dead_code, unused_imports)]
#[cfg(test)]
mod tests
{
	use core::{iter, time::Duration};

	use axum_test_helper::TestClient;
	use casbin::{CoreApi, Enforcer};
	use csv::WriterBuilder;
	use futures::{stream, FutureExt, StreamExt, TryFutureExt};
	use mockd::{address, company, contact, internet, job, name, password, words};
	use money2::{Exchange, ExchangeRates};
	use sqlx::Pool;
	use test_client_ext::{Method, TestClientExt};
	use tracing_test::traced_test;
	use winvoice_adapter::{
		schema::{
			ContactAdapter,
			DepartmentAdapter,
			EmployeeAdapter,
			ExpensesAdapter,
			JobAdapter,
			LocationAdapter,
			OrganizationAdapter,
			TimesheetAdapter,
		},
		Deletable,
		Retrievable,
		Updatable,
	};
	use winvoice_match::{
		Match,
		MatchContact,
		MatchDepartment,
		MatchEmployee,
		MatchExpense,
		MatchJob,
		MatchLocation,
		MatchOrganization,
		MatchTimesheet,
	};
	use winvoice_schema::{
		chrono,
		chrono::{DateTime, TimeZone, Utc},
		ContactKind,
		Currency,
		Invoice,
		Location,
		Money,
	};

	#[allow(clippy::wildcard_imports)]
	use super::*;
	use crate::{
		api::{
			request,
			response::{Get, Login, Logout, Version},
			Code,
			Status,
		},
		lock,
		permissions::{Action, Object},
		r#match::{MatchRole, MatchUser},
		schema::{RoleAdapter, UserAdapter},
		server::response::{LoginResponse, LogoutResponse, Response},
		utils,
	};

	const DEFAULT_SESSION_TTL: Duration = Duration::from_secs(60 * 2);
	const DEFAULT_TIMEOUT: Option<Duration> = Some(Duration::from_secs(60 * 3));

	/// The fields for an [`Contact`](winvoice_schema::Contact)
	fn contact_args() -> (ContactKind, String)
	{
		(ContactKind::Email(contact::email()), words::sentence(4))
	}

	/// The fields for an [`Employee`](winvoice_schema::Expense) (without the [`Department`].
	fn employee_args() -> (String, String)
	{
		(name::full(), job::title())
	}

	/// The fields for an [`Expense`](winvoice_schema::Expense)
	fn expense_args() -> (String, Money, String)
	{
		(words::word(), Money::new(20_00, 2, utils::rand_currency()), words::sentence(5))
	}

	fn job_args() -> (Option<DateTime<Utc>>, DateTime<Utc>, Duration, Invoice, String, String)
	{
		(
			None,
			Utc::now(),
			Duration::new(7640, 0),
			Invoice { date: None, hourly_rate: Money::new(20_38, 2, utils::rand_currency()) },
			words::sentence(5),
			words::sentence(5),
		)
	}

	/// The fields for a [`Location`]
	fn location_args() -> (Option<Currency>, String, Option<Location>)
	{
		(utils::rand_currency().into(), address::country(), None)
	}

	/// The fields for a [`Role`]
	fn role_args() -> (String, Option<Duration>)
	{
		(words::sentence(5), Duration::from_secs(rand::random::<u16>().into()).into())
	}

	/// The fields for a [`Timesheet`](winvoice_schema::Timesheet)
	#[allow(clippy::type_complexity)]
	fn timesheet_args() -> (Vec<(String, Money, String)>, DateTime<Utc>, Option<DateTime<Utc>>, String)
	{
		(
			Default::default(),
			Utc.with_ymd_and_hms(2022, 6, 8, 15, 27, 0).unwrap(),
			Utc.with_ymd_and_hms(2022, 6, 9, 7, 0, 0).latest(),
			words::sentence(5),
		)
	}

	#[allow(unused_macros)]
	macro_rules! fn_setup {
		($Adapter:ty, $Db:ty, $connect:path, $rand_department_name:path) => {
			/// Setup for the tests.
			///
			/// # Returns
			///
			/// * `(client, pool, admin, admin_password, guest, guest_password)`
			async fn setup(test: &str, session_ttl: Duration, time_out: Option<Duration>) -> DynResult<TestData<$Db>>
			{
				let admin_role_name = words::sentence(5);
				let grunt_role_name = words::sentence(5);
				let manager_role_name = words::sentence(5);

				let policy = {
					let mut policy_csv = WriterBuilder::new().has_headers(false).from_writer(Vec::new());
					{
						let mut write = |role: &str, obj: Object| -> csv::Result<()> {
							policy_csv.serialize(("p", role, obj, Action::Create))?;
							policy_csv.serialize(("p", role, obj, Action::Delete))?;
							policy_csv.serialize(("p", role, obj, Action::Retrieve))?;
							policy_csv.serialize(("p", role, obj, Action::Update))?;
							Ok(())
						};

						{
							let mut admin = |obj: Object| -> csv::Result<()> { write(&admin_role_name, obj) };
							admin(Object::Contact)?;
							admin(Object::Department)?;
							admin(Object::Employee)?;
							admin(Object::Expenses)?;
							admin(Object::Job)?;
							admin(Object::Location)?;
							admin(Object::Organization)?;
							admin(Object::Role)?;
							admin(Object::Timesheet)?;
							admin(Object::User)?;
						}

						{
							let mut grunt = |obj: Object| -> csv::Result<()> { write(&grunt_role_name, obj) };
							grunt(Object::CreatedExpenses)?;
							grunt(Object::CreatedTimesheet)?;
						}

						{
							let mut manager = |obj: Object| -> csv::Result<()> { write(&manager_role_name, obj) };
							manager(Object::AssignedDepartment)?;
							manager(Object::EmployeeInDepartment)?;
							manager(Object::ExpensesInDepartment)?;
							manager(Object::JobInDepartment)?;
							manager(Object::TimesheetInDepartment)?;
							manager(Object::UserInDepartment)?;
						}
					}

					policy_csv.serialize(("p", &grunt_role_name, Object::EmployeeSelf, Action::Retrieve))?;
					policy_csv.serialize(("p", &grunt_role_name, Object::EmployeeSelf, Action::Update))?;
					policy_csv.serialize(("p", &grunt_role_name, Object::UserSelf, Action::Retrieve))?;
					policy_csv.serialize(("p", &grunt_role_name, Object::UserSelf, Action::Update))?;

					let inner = policy_csv.into_inner()?;
					String::from_utf8(inner)?
				};

				tracing::debug!("Generated policy: {policy}");

				let (model_path, policy_path) = utils::init_model_and_policy_files(
					&format!("server::{}::{test}", stringify!($Adapter)),
					utils::Model::Rbac.to_string(),
					policy,
				)
				.await
				.map(|(m, p)| {
					(utils::leak_string(m.to_string_lossy().into()), utils::leak_string(p.to_string_lossy().into()))
				})?;

				let enforcer = Enforcer::new(model_path, policy_path).await.map(lock::new)?;

				let pool = $connect();
				let server = Server::<$Adapter>::router(
					None,
					utils::cookie_secret(),
					ServerState::<$Db>::new(enforcer, pool.clone()),
					session_ttl,
					time_out,
				)
				.await?;

				let admin_password = password::generate(true, true, true, 8);
				let grunt_password = password::generate(true, true, true, 8);
				let guest_password = password::generate(true, true, true, 8);
				let manager_password = password::generate(true, true, true, 8);
				let manager_department = <$Adapter as ::winvoice_adapter::schema::Adapter>::Department::create(
					&pool,
					$rand_department_name(),
				)
				.await
				.unwrap();

				#[rustfmt::skip]
				let (admin, grunt, guest, manager) = futures::try_join!(
					<$Adapter as ::winvoice_adapter::schema::Adapter>::Department::create(&pool,
						$rand_department_name()
					).and_then(|department|
						<$Adapter as ::winvoice_adapter::schema::Adapter>::Employee::create(&pool,
							department, name::full(), job::title(),
						).and_then(|employee| <$Adapter as Adapter>::Role::create(&pool,
							admin_role_name, Duration::from_secs(60).into(),
						).and_then(|role| <$Adapter as Adapter>::User::create(&pool,
							employee.into(), admin_password.to_owned(), role, internet::username(),
						)))
					),

					<$Adapter as ::winvoice_adapter::schema::Adapter>::Employee::create(&pool,
						manager_department.clone(), name::full(), job::title(),
					).and_then(|employee| <$Adapter as Adapter>::Role::create(&pool,
						grunt_role_name, Duration::from_secs(60).into(),
					).and_then(|role| <$Adapter as Adapter>::User::create(&pool,
						employee.into(), grunt_password.to_owned(), role, internet::username(),
					))),

					<$Adapter as ::winvoice_adapter::schema::Adapter>::Department::create(&pool,
						$rand_department_name()
					).and_then(|department|
						<$Adapter as ::winvoice_adapter::schema::Adapter>::Employee::create(&pool,
							department, name::full(), job::title(),
						).and_then(|employee| <$Adapter as Adapter>::Role::create(&pool,
							words::sentence(5), Duration::from_secs(60).into(),
						).and_then(|role| <$Adapter as Adapter>::User::create(&pool,
							employee.into(), guest_password.to_owned(), role, internet::username(),
						)))
					),

					<$Adapter as ::winvoice_adapter::schema::Adapter>::Employee::create(&pool,
						manager_department, name::full(), job::title(),
					).and_then(|employee| <$Adapter as Adapter>::Role::create(&pool,
						manager_role_name, Duration::from_secs(60).into(),
					).and_then(|role| <$Adapter as Adapter>::User::create(&pool,
						employee.into(), manager_password.to_owned(), role, internet::username(),
					))),
				)?;

				Ok(TestData {
					client: TestClient::new(server),
					pool,
					admin: (admin, admin_password),
					grunt: (grunt, grunt_password),
					guest: (guest, guest_password),
					manager: (manager, manager_password),
				})
			}

			#[tokio::test]
			#[traced_test]
			async fn rejections() -> DynResult<()>
			{
				let TestData { client, admin, .. } = setup("rejections", DEFAULT_SESSION_TTL, DEFAULT_TIMEOUT).await?;

				#[rustfmt::skip]
			                stream::iter([
					routes::CONTACT, routes::EMPLOYEE, routes::EXPENSE, routes::JOB, routes::LOCATION,
					routes::LOGOUT, routes::ORGANIZATION, routes::ROLE, routes::TIMESHEET, routes::USER,
				])
				.for_each(|route| async {
					tracing::debug!(r#"Testing "{}" rejections…"#, &*route);

					{// assert request rejected when no API version header.
						let response = client.get(route).send().await;
						assert_eq!(response.status(), StatusCode::from(Code::ApiVersionHeaderMissing));
						assert_eq!(&response.json::<Version>().await, VersionResponse::missing().content());
					}

					if route.ne(routes::LOGOUT)
					{
						{// assert GETs w/out login are rejected
							let response = client.get_builder(route).send().await;
							assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
						}

						{// assert GETs w/ wrong body are rejected
							client.login(admin.0.username(), &admin.1).await;

							let response = client.get_builder(route).body("").send().await;
							assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);

							client.logout().await;
						}
					}
				})
				.await;

				Ok(())
			}
		};
	}

	/// Data used for tests.
	struct TestData<Db>
	where
		Db: Database,
	{
		/// A user with every top-level permissions.
		admin: (User, String),

		/// An HTTP client which can be used to communicate with a local instance of the winvoice server.
		client: TestClient,

		/// A user with mid-level permissions.
		manager: (User, String),

		/// A user with bottom-level permissions.
		grunt: (User, String),

		/// A user with no permissions.
		guest: (User, String),

		/// A connection to the database.
		pool: Pool<Db>,
	}

	#[cfg(feature = "test-postgres")]
	mod postgres
	{
		use pretty_assertions::assert_eq;
		use sqlx::Postgres;
		use winvoice_adapter_postgres::{
			schema::{
				util::{connect, rand_department_name},
				PgContact,
				PgDepartment,
				PgEmployee,
				PgExpenses,
				PgJob,
				PgLocation,
				PgOrganization,
				PgTimesheet,
			},
			PgSchema,
		};

		#[allow(clippy::wildcard_imports)]
		use super::*;
		use crate::schema::postgres::{PgRole, PgUser};

		fn_setup!(PgSchema, Postgres, connect, rand_department_name);

		#[tokio::test]
		#[traced_test]
		async fn delete() -> DynResult<()>
		{
			let TestData { admin, client, grunt, guest, manager, pool } =
				setup("employee_get", DEFAULT_SESSION_TTL, DEFAULT_TIMEOUT).await?;

			macro_rules! check {
                ($Adapter:ty, $route:ident; $($pass:ident: $data:expr => $code:expr),+; $($fail:ident),+) => {
                    stream::iter([$((&$pass, &$data, $code)),+]).for_each(|data| client.test_other_success::<$Adapter>(
                        Method::Delete,
                        &pool,
                        routes::$route,
                        &data.0.0,
                        &data.0.1,
                        vec![data.1.clone()],
                        data.2,
                    ))
                    .await;

                    stream::iter([$(&$fail),+]).for_each(|data|
                        client.test_other_unauthorized(Method::Delete, routes::$route, &data.0, &data.1)
                    )
                    .await;
                }
            }

			let contact_ = {
				let (kind, label) = contact_args();
				PgContact::create(&pool, kind, label).await?
			};

			let department = PgDepartment::create(&pool, rand_department_name()).await?;

			let employee = {
				let (name_, title) = employee_args();
				PgEmployee::create(&pool, manager.0.department().unwrap().clone(), name_, title).await?
			};

			let location = {
				let (currency, address_, outer) = location_args();
				PgLocation::create(&pool, currency, address_, outer).await?
			};

			let organization = PgOrganization::create(&pool, location.clone(), company::company()).await?;

			let rates = ExchangeRates::new().await?;

			let [job_, job2]: [_; 2] = {
				let mut tx = pool.begin().await?;
				let (date_close, date_open, increment, invoice, notes, objectives) = job_args();
				let j = PgJob::create(
					&mut tx,
					organization.clone(),
					date_close,
					date_open,
					[department.clone()].into_iter().collect(),
					increment,
					invoice,
					notes,
					objectives,
				)
				.await?;

				let (date_close, date_open, increment, invoice, notes, objectives) = job_args();
				let j2 = PgJob::create(
					&mut tx,
					organization.clone(),
					date_close,
					date_open,
					manager.0.employee().into_iter().map(|e| e.department.clone()).collect(),
					increment,
					invoice,
					notes,
					objectives,
				)
				.await?;

				tx.commit().await?;
				[j, j2]
					.into_iter()
					.map(|jo| jo.exchange(Default::default(), &rates))
					.collect::<Vec<_>>()
					.try_into()
					.unwrap()
			};

			let [timesheet, timesheet2, timesheet3]: [_; 3] = {
				let mut tx = pool.begin().await?;
				let (expenses, time_begin, time_end, work_notes) = timesheet_args();
				let t = PgTimesheet::create(
					&mut tx,
					employee.clone(),
					expenses,
					job_.clone(),
					time_begin,
					time_end,
					work_notes,
				)
				.await?;

				let (expenses, time_begin, time_end, work_notes) = timesheet_args();
				let t2 = PgTimesheet::create(
					&mut tx,
					grunt.0.employee().unwrap().clone(),
					expenses,
					job2.clone(),
					time_begin,
					time_end,
					work_notes,
				)
				.await?;

				let (expenses, time_begin, time_end, work_notes) = timesheet_args();
				let t3 = PgTimesheet::create(
					&mut tx,
					manager.0.employee().unwrap().clone(),
					expenses,
					job2.clone(),
					time_begin,
					time_end,
					work_notes,
				)
				.await?;

				tx.commit().await?;
				[t, t2, t3]
					.into_iter()
					.map(|ts| ts.exchange(Default::default(), &rates))
					.collect::<Vec<_>>()
					.try_into()
					.unwrap()
			};

			let expenses = {
				let mut x = Vec::with_capacity(3);
				for t in [&timesheet, &timesheet2, &timesheet3]
				{
					PgExpenses::create(&pool, iter::repeat_with(expense_args).take(1).collect(), t.id)
						.await
						.map(|mut v| x.append(&mut v))?;
				}

				x.exchange(Default::default(), &rates)
			};

			let role = {
				let (name_, password_ttl) = role_args();
				PgRole::create(&pool, name_, password_ttl).await?
			};

			let user = PgUser::create(
				&pool,
				None,
				password::generate(true, true, true, 8),
				role.clone(),
				internet::username(),
			)
			.await?;

			let manager_user = PgUser::create(
				&pool,
				employee.clone().into(),
				password::generate(true, true, true, 8),
				role.clone(),
				internet::username(),
			)
			.await?;

			let users = [&admin.0, &guest.0, &grunt.0, &manager.0].into_iter().cloned().collect::<Vec<_>>();
			let roles = users.iter().map(User::role).collect::<Vec<_>>();

			check!(
				PgUser, USER;
				admin: user => None,
				manager: manager_user => Code::SuccessForPermissions.into();
				grunt,
				guest
			);
			check!(PgRole, ROLE; admin: role => None; grunt, guest, manager);
			check!(
				PgExpenses, EXPENSE;
				admin: expenses[2] => None,
				grunt: expenses[1] => Code::SuccessForPermissions.into(),
				manager: expenses[0] => Code::SuccessForPermissions.into();
				guest
			);

			// TODO: /timesheet
			PgTimesheet::delete(&pool, [&timesheet, &timesheet2, &timesheet3].into_iter()).await?;

			// TODO: /job
			PgJob::delete(&pool, [&job_, &job2].into_iter()).await?;

			// TODO: /employee
			PgEmployee::delete(&pool, [&employee].into_iter()).await?;

			check!(PgOrganization, ORGANIZATION; admin: organization => None; grunt, guest, manager);
			check!(PgContact, CONTACT; admin: contact_ => None; grunt, guest, manager);
			check!(PgLocation, LOCATION; admin: location => None; grunt, guest, manager);

			// TODO: /department

			PgUser::delete(&pool, users.iter()).await?;
			futures::try_join!(
				PgRole::delete(&pool, roles.into_iter()),
				PgEmployee::delete(&pool, users.iter().filter_map(User::employee)),
			)?;
			PgDepartment::delete(&pool, users.iter().filter_map(|u| u.employee().map(|e| &e.department))).await?;

			Ok(())
		}

		#[tokio::test]
		#[traced_test]
		async fn get() -> DynResult<()>
		{
			let TestData { admin, client, grunt, guest, manager, pool } =
				setup("employee_get", DEFAULT_SESSION_TTL, DEFAULT_TIMEOUT).await?;

			macro_rules! assert_unauthorized {
                ($Match:ty, $route:ident; $($fail:ident),+) => {
                    stream::iter([$(&$fail),+]).for_each(|data|
                        client.test_get_unauthorized::<$Match>(routes::$route, &data.0, &data.1)
                    )
                    .await;
                }
            }

			let contact_ = {
				let (kind, label) = contact_args();
				PgContact::create(&pool, kind, label).await?
			};

			#[rustfmt::skip]
			client.test_get_success(
				routes::CONTACT,
				&admin.0, &admin.1,
				MatchContact::from(contact_.label.clone()),
				[&contact_].into_iter(), None,
			)
			.await;
			assert_unauthorized!(MatchContact, CONTACT; guest, grunt, manager);

			let department = PgDepartment::create(&pool, rand_department_name()).await?;

			#[rustfmt::skip]
			client.test_get_success(
				routes::DEPARTMENT,
				&admin.0, &admin.1,
				MatchDepartment::from(department.id),
				[&department].into_iter(), None,
			)
			.then(|_| client.test_get_success(
				routes::DEPARTMENT,
				&manager.0, &manager.1,
				MatchDepartment::default(),
				manager.0.employee().into_iter().map(|e| &e.department), Code::SuccessForPermissions.into(),
			))
			.await;
			assert_unauthorized!(MatchDepartment, DEPARTMENT; guest, grunt);

			let employee = {
				let (name_, title) = employee_args();
				PgEmployee::create(&pool, department.clone(), name_, title).await?
			};

			#[rustfmt::skip]
			client.test_get_success(
				routes::EMPLOYEE,
				&admin.0, &admin.1,
				MatchEmployee::from(employee.id),
				[&employee].into_iter(), None,
			)
			.then(|_| client.test_get_success(
				routes::EMPLOYEE,
				&grunt.0, &grunt.1,
				MatchEmployee::default(),
				grunt.0.employee().into_iter(), Code::SuccessForPermissions.into(),
			))
			.then(|_| client.test_get_success(
				routes::EMPLOYEE,
				&manager.0, &manager.1,
				MatchEmployee::default(),
				[&grunt, &manager].into_iter().map(|e| e.0.employee().unwrap()), Code::SuccessForPermissions.into(),
			))
			.await;
			assert_unauthorized!(MatchEmployee, EMPLOYEE; guest);

			let location = {
				let (currency, address_, outer) = location_args();
				PgLocation::create(&pool, currency, address_, outer).await?
			};

			#[rustfmt::skip]
			client.test_get_success(
				routes::LOCATION,
				&admin.0, &admin.1,
				MatchLocation::from(location.id),
				[&location].into_iter(), None,
			)
			.await;
			assert_unauthorized!(MatchLocation, LOCATION; guest, grunt, manager);

			let organization = PgOrganization::create(&pool, location.clone(), company::company()).await?;

			#[rustfmt::skip]
			client.test_get_success(
				routes::ORGANIZATION,
				&admin.0, &admin.1,
				MatchOrganization::from(organization.id),
				[&organization].into_iter(), None,
			)
			.await;
			assert_unauthorized!(MatchOrganization, ORGANIZATION; guest, grunt, manager);

			let rates = ExchangeRates::new().await?;

			let [job_, job2]: [_; 2] = {
				let mut tx = pool.begin().await?;
				let (date_close, date_open, increment, invoice, notes, objectives) = job_args();
				let j = PgJob::create(
					&mut tx,
					organization.clone(),
					date_close,
					date_open,
					[department.clone()].into_iter().collect(),
					increment,
					invoice,
					notes,
					objectives,
				)
				.await?;

				let (date_close, date_open, increment, invoice, notes, objectives) = job_args();
				let j2 = PgJob::create(
					&mut tx,
					organization.clone(),
					date_close,
					date_open,
					manager.0.employee().into_iter().map(|e| e.department.clone()).collect(),
					increment,
					invoice,
					notes,
					objectives,
				)
				.await?;

				tx.commit().await?;
				[j, j2]
					.into_iter()
					.map(|jo| jo.exchange(Default::default(), &rates))
					.collect::<Vec<_>>()
					.try_into()
					.unwrap()
			};

			#[rustfmt::skip]
			client.test_get_success(
				routes::JOB,
				&admin.0, &admin.1,
				MatchJob::from(job_.id),
				[&job_].into_iter(), None,
			)
			.then(|_| client.test_get_success(
				routes::JOB,
				&manager.0, &manager.1,
				MatchJob::default(),
				[&job2].into_iter(), Code::SuccessForPermissions.into(),
			))
			.await;
			assert_unauthorized!(MatchJob, JOB; guest, grunt);

			let [timesheet, timesheet2, timesheet3]: [_; 3] = {
				let mut tx = pool.begin().await?;
				let (expenses, time_begin, time_end, work_notes) = timesheet_args();
				let t = PgTimesheet::create(
					&mut tx,
					employee.clone(),
					expenses,
					job_.clone(),
					time_begin,
					time_end,
					work_notes,
				)
				.await?;

				let (expenses, time_begin, time_end, work_notes) = timesheet_args();
				let t2 = PgTimesheet::create(
					&mut tx,
					grunt.0.employee().unwrap().clone(),
					expenses,
					job2.clone(),
					time_begin,
					time_end,
					work_notes,
				)
				.await?;

				let (expenses, time_begin, time_end, work_notes) = timesheet_args();
				let t3 = PgTimesheet::create(
					&mut tx,
					manager.0.employee().unwrap().clone(),
					expenses,
					job2.clone(),
					time_begin,
					time_end,
					work_notes,
				)
				.await?;

				tx.commit().await?;
				[t, t2, t3]
					.into_iter()
					.map(|ts| ts.exchange(Default::default(), &rates))
					.collect::<Vec<_>>()
					.try_into()
					.unwrap()
			};

			#[rustfmt::skip]
			client.test_get_success(
				routes::TIMESHEET,
				&admin.0, &admin.1,
				MatchTimesheet::from(timesheet.id),
				[&timesheet].into_iter(), None,
			)
			.then(|_| client.test_get_success(
				routes::TIMESHEET,
				&grunt.0, &grunt.1,
				MatchTimesheet::default(),
				[&timesheet2].into_iter(), Code::SuccessForPermissions.into(),
			))
			.then(|_| client.test_get_success(
				routes::TIMESHEET,
				&manager.0, &manager.1,
				MatchTimesheet::default(),
				[&timesheet2, &timesheet3].into_iter(), Code::SuccessForPermissions.into(),
			))
			.await;
			assert_unauthorized!(MatchTimesheet, TIMESHEET; guest);

			let expenses = {
				let mut x = Vec::with_capacity(2 * 3);
				for t in [&timesheet, &timesheet2, &timesheet3]
				{
					PgExpenses::create(&pool, iter::repeat_with(expense_args).take(2).collect(), t.id)
						.await
						.map(|mut v| x.append(&mut v))?;
				}

				x.exchange(Default::default(), &rates)
			};

			#[rustfmt::skip]
			client.test_get_success(
				routes::EXPENSE,
				&admin.0, &admin.1,
				MatchExpense::from(Match::Or(expenses.iter().map(|x| x.id.into()).collect())),
				expenses.iter(), None,
			)
			.then(|_| client.test_get_success(
				routes::EXPENSE,
				&grunt.0, &grunt.1,
				MatchExpense::default(),
				expenses.iter().filter(|x| x.timesheet_id == timesheet2.id), Code::SuccessForPermissions.into(),
			))
			.then(|_| client.test_get_success(
				routes::EXPENSE,
				&manager.0, &manager.1,
				MatchExpense::default(),
				expenses.iter().filter(|x| x.timesheet_id == timesheet2.id || x.timesheet_id == timesheet3.id),
				Code::SuccessForPermissions.into(),
			))
			.await;
			assert_unauthorized!(MatchExpense, EXPENSE; guest);

			let users = serde_json::to_string(&[&admin.0, &guest.0, &grunt.0, &manager.0])
				.and_then(|json| serde_json::from_str::<[User; 4]>(&json))?;

			let roles = users.iter().map(|u| u.role().clone()).collect::<Vec<_>>();

			assert_unauthorized!(MatchRole, ROLE; guest, grunt, manager);
			client
				.test_get_success(
					routes::ROLE,
					&admin.0,
					&admin.1,
					MatchRole::from(Match::Or(roles.iter().map(|r| r.id().into()).collect())),
					roles.iter(),
					None,
				)
				.await;

			#[rustfmt::skip]
			client.test_get_success(
				routes::USER,
				&admin.0, &admin.1,
				MatchUser::from(Match::Or(users.iter().map(|u| u.id().into()).collect())),
				users.iter(), None,
			)
			.then(|_| client.test_get_success(
				routes::USER,
				&grunt.0, &grunt.1,
				MatchUser::default(),
				users.iter().filter(|u| u.id() == grunt.0.id()), Code::SuccessForPermissions.into(),
			))
			.then(|_| client.test_get_success(
				routes::USER,
				&manager.0, &manager.1,
				MatchUser::default(),
				users.iter().filter(|u| u.id() == grunt.0.id() || u.id() == manager.0.id()),
                Code::SuccessForPermissions.into(),
			))
			.await;
			assert_unauthorized!(MatchUser, USER; guest);

			PgUser::delete(&pool, users.iter()).await?;
			futures::try_join!(PgRole::delete(&pool, roles.iter()), PgJob::delete(&pool, [&job_, &job2].into_iter()))?;

			PgOrganization::delete(&pool, [organization].iter()).await?;
			futures::try_join!(
				PgContact::delete(&pool, [&contact_].into_iter()),
				PgEmployee::delete(&pool, users.iter().filter_map(User::employee).chain([&employee])),
				PgLocation::delete(&pool, [&location].into_iter()),
			)?;
			PgDepartment::delete(
				&pool,
				users
					.iter()
					.filter_map(|u| u.employee().map(|e| &e.department))
					.chain([&employee.department, &department]),
			)
			.await?;

			Ok(())
		}

		#[tokio::test]
		#[traced_test]
		async fn patch() -> DynResult<()>
		{
			let TestData { admin, client, grunt, guest, manager, pool } =
				setup("employee_get", DEFAULT_SESSION_TTL, DEFAULT_TIMEOUT).await?;

			macro_rules! check {
                ($Adapter:ty, $route:ident; $($pass:ident: $data:expr => $code:expr),+; $($fail:ident),+) => {
                    stream::iter([$((&$pass, &$data, $code)),+]).for_each(|data| client.test_other_success::<$Adapter>(
                        Method::Patch,
                        &pool,
                        routes::$route,
                        &data.0.0,
                        &data.0.1,
                        vec![data.1.clone()],
                        data.2,
                    ))
                    .await;

                    stream::iter([$(&$fail),+]).for_each(|data|
                        client.test_other_unauthorized(Method::Delete, routes::$route, &data.0, &data.1)
                    )
                    .await;
                }
            }

			let contact_ = {
				let (kind, label) = contact_args();
				PgContact::create(&pool, kind, label).await.map(|mut c| {
					c.kind = ContactKind::Other(format!("@{}", internet::username()));
					c
				})?
			};

			let department = PgDepartment::create(&pool, rand_department_name()).await.map(|mut d| {
				d.name = words::sentence(7);
				d
			})?;

			let employee = {
				let (name_, title) = employee_args();
				PgEmployee::create(&pool, department.clone(), name_, title).await.map(|mut e| {
					e.name = name::full();
					e
				})?
			};

			let location = {
				let (currency, address_, outer) = location_args();
				PgLocation::create(&pool, currency, address_, outer).await.map(|mut l| {
					l.name = address::street();
					l
				})?
			};

			let organization =
				PgOrganization::create(&pool, location.clone(), company::company()).await.map(|mut o| {
					o.name = words::sentence(4);
					o
				})?;

			let rates = ExchangeRates::new().await?;

			let [job_, job2]: [_; 2] = {
				let mut tx = pool.begin().await?;
				let (date_close, date_open, increment, invoice, notes, objectives) = job_args();
				let j = PgJob::create(
					&mut tx,
					organization.clone(),
					date_close,
					date_open,
					[department.clone()].into_iter().collect(),
					increment,
					invoice,
					notes,
					objectives,
				)
				.await
				.map(|mut j| {
					j.date_close = (j.date_open + chrono::Duration::days(30)).into();
					j
				})?;

				let (date_close, date_open, increment, invoice, notes, objectives) = job_args();
				let j2 = PgJob::create(
					&mut tx,
					organization.clone(),
					date_close,
					date_open,
					manager.0.employee().into_iter().map(|e| e.department.clone()).collect(),
					increment,
					invoice,
					notes,
					objectives,
				)
				.await
				.map(|mut j| {
					j.date_close = (j.date_open + chrono::Duration::days(30)).into();
					j
				})?;

				tx.commit().await?;
				[j, j2]
					.into_iter()
					.map(|jo| jo.exchange(Default::default(), &rates))
					.collect::<Vec<_>>()
					.try_into()
					.unwrap()
			};

			let [timesheet, timesheet2, timesheet3]: [_; 3] = {
				let mut tx = pool.begin().await?;
				let (expenses, time_begin, time_end, work_notes) = timesheet_args();
				let t = PgTimesheet::create(
					&mut tx,
					employee.clone(),
					expenses,
					job_.clone(),
					time_begin,
					time_end,
					work_notes,
				)
				.await
				.map(|mut t| {
					t.time_end = (t.time_begin + chrono::Duration::hours(3)).into();
					t
				})?;

				let (expenses, time_begin, time_end, work_notes) = timesheet_args();
				let t2 = PgTimesheet::create(
					&mut tx,
					grunt.0.employee().unwrap().clone(),
					expenses,
					job2.clone(),
					time_begin,
					time_end,
					work_notes,
				)
				.await
				.map(|mut t| {
					t.time_end = (t.time_begin + chrono::Duration::hours(3)).into();
					t
				})?;

				let (expenses, time_begin, time_end, work_notes) = timesheet_args();
				let t3 = PgTimesheet::create(
					&mut tx,
					manager.0.employee().unwrap().clone(),
					expenses,
					job2.clone(),
					time_begin,
					time_end,
					work_notes,
				)
				.await?;

				tx.commit().await?;
				[t, t2, t3]
					.into_iter()
					.map(|ts| ts.exchange(Default::default(), &rates))
					.collect::<Vec<_>>()
					.try_into()
					.unwrap()
			};

			let expenses = {
				let mut x = Vec::with_capacity(2 * 3);
				for t in [&timesheet, &timesheet2, &timesheet3]
				{
					PgExpenses::create(&pool, iter::repeat_with(expense_args).take(2).collect(), t.id).await.map(
						|v| {
							x.extend(v.into_iter().map(|mut x| {
								x.category = words::sentence(3);
								x
							}))
						},
					)?;
				}

				x.exchange(Default::default(), &rates)
			};

			let role = {
				let (name_, password_ttl) = role_args();
				PgRole::create(&pool, name_, password_ttl).await?
			};

			let user = PgUser::create(
				&pool,
				None,
				password::generate(true, true, true, 8),
				role.clone(),
				internet::username(),
			)
			.await
			.map(|mut u| {
				u.employee = employee.clone().into();
				u
			})?;

			let users = [&admin.0, &guest.0, &grunt.0, &manager.0].into_iter().cloned().collect::<Vec<_>>();
			let roles = users.iter().map(User::role).collect::<Vec<_>>();

			// TODO: /user

			check!(PgRole, ROLE; admin: role => None; grunt, guest, manager);

			// TODO: /expense
			// TODO: /timesheet
			// TODO: /job

			check!(PgOrganization, ORGANIZATION; admin: organization => None; grunt, guest, manager);
			check!(PgContact, CONTACT; admin: contact_ => None; grunt, guest, manager);
			check!(PgLocation, LOCATION; admin: location => None; grunt, guest, manager);

			// TODO: /department

			PgUser::delete(&pool, users.iter().chain([&user])).await?;
			futures::try_join!(
				PgContact::delete(&pool, [&contact_].into_iter()),
				PgJob::delete(&pool, [&job_, &job2].into_iter()),
				PgRole::delete(&pool, roles.into_iter().chain([&role])),
			)?;

			PgOrganization::delete(&pool, [&organization].into_iter()).await?;
			futures::try_join!(
				PgDepartment::delete(&pool, users.iter().filter_map(|u| u.employee().map(|e| &e.department))),
				PgEmployee::delete(&pool, users.iter().filter_map(User::employee)),
				PgLocation::delete(&pool, [&location].into_iter()),
			)?;

			Ok(())
		}

		#[tokio::test]
		#[traced_test]
		async fn post() -> DynResult<()>
		{
			let TestData { admin, client, grunt, guest, manager, pool } =
				setup("employee_get", DEFAULT_SESSION_TTL, DEFAULT_TIMEOUT).await?;

			client.test_post_unauthorized(&pool, routes::CONTACT, &grunt.0, &grunt.1, contact_args()).await;
			client.test_post_unauthorized(&pool, routes::CONTACT, &guest.0, &guest.1, contact_args()).await;
			client.test_post_unauthorized(&pool, routes::CONTACT, &manager.0, &manager.1, contact_args()).await;
			let contact_ = client
				.test_post_success::<PgContact, _>(&pool, routes::CONTACT, &admin.0, &admin.1, contact_args(), None)
				.await;

			// TODO: /department
			// TODO: /employee

			client.test_post_unauthorized(&pool, routes::LOCATION, &grunt.0, &grunt.1, location_args()).await;
			client.test_post_unauthorized(&pool, routes::LOCATION, &guest.0, &guest.1, location_args()).await;
			client.test_post_unauthorized(&pool, routes::LOCATION, &manager.0, &manager.1, location_args()).await;
			let location = client
				.test_post_success::<PgLocation, _>(&pool, routes::LOCATION, &admin.0, &admin.1, location_args(), None)
				.await;

			let organization_args = || (location.clone(), words::sentence(5));

			client.test_post_unauthorized(&pool, routes::ORGANIZATION, &grunt.0, &grunt.1, organization_args()).await;
			client.test_post_unauthorized(&pool, routes::ORGANIZATION, &guest.0, &guest.1, organization_args()).await;
			client
				.test_post_unauthorized(&pool, routes::ORGANIZATION, &manager.0, &manager.1, organization_args())
				.await;
			let organization = client
				.test_post_success::<PgOrganization, _>(
					&pool,
					routes::ORGANIZATION,
					&admin.0,
					&admin.1,
					organization_args(),
					None,
				)
				.await;

			let rates = ExchangeRates::new().await?;

			// TODO: /job
			// TODO: /timesheet
			// TODO: /expenses

			client.test_post_unauthorized(&pool, routes::ROLE, &grunt.0, &grunt.1, role_args()).await;
			client.test_post_unauthorized(&pool, routes::ROLE, &guest.0, &guest.1, role_args()).await;
			client.test_post_unauthorized(&pool, routes::ROLE, &manager.0, &manager.1, role_args()).await;
			let role =
				client.test_post_success::<PgRole, _>(&pool, routes::ROLE, &admin.0, &admin.1, role_args(), None).await;

			// TODO: /user

			let users = [&admin.0, &guest.0, &grunt.0, &manager.0].into_iter().cloned().collect::<Vec<_>>();
			let roles = users.iter().map(User::role).chain([&role]).collect::<Vec<_>>();

			PgUser::delete(&pool, users.iter()).await?;
			futures::try_join!(
				PgRole::delete(&pool, roles.into_iter()),
				/* PgJob::delete(&pool, [&job_, &job2].into_iter()) */
			)?;

			PgOrganization::delete(&pool, [organization].iter()).await?;
			futures::try_join!(
				PgContact::delete(&pool, [&contact_].into_iter()),
				// PgEmployee::delete(&pool, [&employee].into_iter()),
				PgLocation::delete(&pool, [&location].into_iter()),
			)?;

			Ok(())
		}
	}
}
