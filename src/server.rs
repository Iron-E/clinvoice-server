//! The `server` module functions to spawn an [`axum_server`] which communicates over TLS.

mod auth;
mod db_session_store;
mod response;
mod state;

use core::{fmt::Display, marker::PhantomData, time::Duration};
use std::{collections::HashSet, net::SocketAddr};

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use auth::{AuthContext, DbUserStore, InitializableWithAuthorization, RequireAuthLayer, UserStore};
use axum::{
	error_handling::HandleErrorLayer,
	extract::{Extension, Json, State},
	headers::{authorization::Basic, Authorization},
	http::{HeaderMap, Request, StatusCode},
	middleware::{self, Next},
	response::IntoResponse,
	routing,
	BoxError,
	Router,
	TypedHeader,
};
use axum_login::{
	axum_sessions::{async_session::SessionStore, SessionLayer},
	AuthLayer,
	SqlxStore,
};
use axum_server::tls_rustls::RustlsConfig;
use db_session_store::DbSessionStore;
pub use response::{LoginResponse, LogoutResponse, Response, VersionResponse};
use semver::VersionReq;
use sqlx::{Connection, Database, Executor, QueryBuilder, Transaction};
pub use state::ServerState;
use tower::{timeout, ServiceBuilder};
use tower_http::{compression::CompressionLayer, trace::TraceLayer};
use winvoice_adapter::{
	fmt::sql,
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
	Initializable,
	Retrievable,
	Updatable,
};
use winvoice_match::{Match, MatchDepartment, MatchEmployee, MatchExpense, MatchJob, MatchOption, MatchTimesheet};
use winvoice_schema::{chrono::Utc, Department, Employee, Expense, Job, Timesheet};

use crate::{
	api::{self, request, response::Retrieve, routes, Code, Status},
	bool_ext::BoolExt,
	permissions::{Action, Object},
	r#match::MatchUser,
	schema::{columns::UserColumns, Adapter, RoleAdapter, User, UserAdapter},
	DynResult,
};

/// Create routes which are able to be implemented generically.
macro_rules! route {
	($Entity:ident) => {
		routing::delete(|| async move { todo("Delete method not implemented") })
			.get(
				|Extension(user): Extension<User>,
				 State(state): State<ServerState<A::Db>>,
				 Json(request): Json<request::Retrieve<<A::$Entity as Retrievable>::Match>>| async move {
					state
						.enforce_permission::<Retrieve<<A::$Entity as Retrievable>::Entity>>(
							&user,
							Object::$Entity,
							Action::Retrieve,
						)
						.await?;

					let condition = request.into_condition();
					A::$Entity::retrieve(state.pool(), condition).await.map_or_else(
						|e| {
							Err(Response::from(Retrieve::<<A::$Entity as Retrievable>::Entity>::from(Status::from(&e))))
						},
						|vec| Ok(Response::from(Retrieve::new(vec, Code::Success.into()))),
					)
				},
			)
			.patch(|| async move { todo("Update method not implemented") })
	};
}

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
	for<'connection> &'connection mut Transaction<'connection, A::Db>: Executor<'connection, Database = A::Db>,
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
				tracing::error!("{e}");
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

		let mut router = Router::new()
			.route(routes::CONTACT, route!(Contact).post(|| async move { todo("contact create") }))
			.route(
				routes::DEPARTMENT,
				routing::delete(|| async move { todo("Delete method not implemented") })
					.get(
						|Extension(user): Extension<User>,
						 State(state): State<ServerState<A::Db>>,
						 Json(request): Json<request::Retrieve<MatchDepartment>>| async move {
							let mut condition = request.into_condition();
							let code = match state.department_permissions(&user, Action::Retrieve).await?
							{
								Object::Department => Code::Success,

								// HACK: no if-let guards…
								Object::AssignedDepartment if user.employee().is_some() =>
								{
									condition.id &= user.employee().unwrap().department.id.into();
									Code::SuccessForPermissions
								},

								// they have no department, so they *effectively* can't retrieve departments.
								Object::AssignedDepartment =>
								{
									return Ok(Response::from(Retrieve::new(
										Default::default(),
										Code::SuccessForPermissions.into(),
									)));
								},

								p => p.unreachable(),
							};

							A::Department::retrieve(state.pool(), condition).await.map_or_else(
								|e| Err(Response::from(Retrieve::<Department>::from(Status::from(&e)))),
								|vec| Ok(Response::from(Retrieve::new(vec, code.into()))),
							)
						},
					)
					.patch(|| async move { todo("Update method not implemented") })
					.post(|| async move { todo("department create") }),
			)
			.route(
				routes::EMPLOYEE,
				routing::delete(|| async move { todo("Delete method not implemented") })
					.get(
						|Extension(user): Extension<User>,
						 State(state): State<ServerState<A::Db>>,
						 Json(request): Json<request::Retrieve<MatchEmployee>>| async move {
							let mut condition = request.into_condition();
							#[rustfmt::skip]
							let code = match state.employee_permissions::<Retrieve<Employee>>(&user, Action::Retrieve).await?
							{
								Some(Object::Employee) => Code::Success,

								// HACK: no if-let guards…
								Some(Object::EmployeeInDepartment) if user.employee().is_some() =>
								{
									condition.department.id &= user.employee().unwrap().department.id.into();
									Code::SuccessForPermissions
								},

								// HACK: no if-let guards…
								None if user.employee().is_some() =>
								{
									condition.id &= user.employee().unwrap().id.into();
									Code::SuccessForPermissions
								},

								Some(Object::EmployeeInDepartment) | None =>
								{
									return Ok(Response::from(Retrieve::new(
										Default::default(),
										Code::SuccessForPermissions.into(),
									)));
								},

								Some(p) => p.unreachable(),
							};

							A::Employee::retrieve(state.pool(), condition).await.map_or_else(
								|e| Err(Response::from(Retrieve::<Employee>::from(Status::from(&e)))),
								|vec| Ok(Response::from(Retrieve::new(vec, code.into()))),
							)
						},
					)
					.patch(|| async move { todo("Update method not implemented") })
					.post(|| async move { todo("employee create") }),
			)
			.route(
				routes::EXPENSE,
				routing::delete(|| async move { todo("Delete method not implemented") })
					.get(
						|Extension(user): Extension<User>,
						 State(state): State<ServerState<A::Db>>,
						 Json(request): Json<request::Retrieve<MatchExpense>>| async move {
							let permission =
								state.expense_permissions::<Retrieve<Expense>>(&user, Action::Retrieve).await?;

							// The user has no department, and no employee record, so they effectively cannot retrieve
							// expenses.
							if permission != Object::Expenses && user.employee().is_none()
							{
								return Ok(Response::from(Retrieve::new(
									Default::default(),
									Code::SuccessForPermissions.into(),
								)));
							}

							let condition = request.into_condition();

							#[rustfmt::skip]
							let mut vec = A::Expenses::retrieve(state.pool(), condition).await.map_err(
								|e| Response::from(Retrieve::<Expense>::from(Status::from(&e))),
							)?;

							#[rustfmt::skip]
							let code = match permission
							{
								Object::Expenses => Code::Success,

								// The user can only get expenses iff they are in the same department, or were created
								// by that user.
								p =>
								{
									match user.employee()
									{
										Some(emp) =>
										{
											#[rustfmt::skip]
											// retrieve IDs of expenses which the user has permission to access.
											// NOTE: `Timesheet::retrieve` retrieves *ALL* expenses for a timesheet, not just the
											//       ones which match the `expenses` field. Thus we still have to perform a second
											//       filter below.
											let matching = A::Timesheet::retrieve(state.pool(), MatchTimesheet {
												expenses: MatchExpense {
													id: Match::Or(vec.iter().map(|x| x.id.into()).collect()),
													..Default::default()
												}
												.into(),
												..match p
												{
													Object::ExpensesInDepartment =>
														MatchJob::from(MatchDepartment::from(emp.department.id)).into(),
													Object::CreatedExpenses => MatchEmployee::from(emp.id).into(),
													_ => p.unreachable(),
												}
											})
											.await
											.map_or_else(
												|e| Err(Response::from(Retrieve::from(Status::from(&e)))),
												|vec| Ok(vec
															.into_iter()
															.flat_map(|t| t.expenses.into_iter().map(|x| x.id))
															.collect::<HashSet<_>>()),
											)?;

											vec.retain(|x| matching.contains(&x.id));
										},

										None => unreachable!("Should have been returned earlier for {permission:?}"),
									};

									Code::SuccessForPermissions
								},
							};

							Ok::<_, Response<_>>(Response::from(Retrieve::new(vec, code.into())))
						},
					)
					.patch(|| async move { todo("Update method not implemented") })
					.post(|| async move { todo("expense create") }),
			)
			.route(
				routes::JOB,
				routing::delete(|| async move { todo("Delete method not implemented") })
					.get(
						|Extension(user): Extension<User>,
						 State(state): State<ServerState<A::Db>>,
						 Json(request): Json<request::Retrieve<MatchJob>>| async move {
							let mut condition = request.into_condition();

							#[rustfmt::skip]
							let code = match state.job_permissions(&user, Action::Retrieve).await?
							{
								Object::Job => Code::Success,

								// HACK: no if-let guards…
								Object::JobInDepartment if user.employee().is_some() =>
								{
									condition.departments &=
										MatchDepartment::from(user.employee().unwrap().department.id).into();

									Code::SuccessForPermissions
								},

								Object::JobInDepartment =>
								{
									return Ok(Response::from(Retrieve::new(
										Default::default(),
										Code::SuccessForPermissions.into(),
									)));
								},

								p => p.unreachable(),
							};

							A::Job::retrieve(state.pool(), condition).await.map_or_else(
								|e| Err(Response::from(Retrieve::<Job>::from(Status::from(&e)))),
								|vec| Ok(Response::from(Retrieve::new(vec, code.into()))),
							)
						},
					)
					.patch(|| async move { todo("Update method not implemented") })
					.post(|| async move { todo("job create") }),
			)
			.route(routes::LOCATION, route!(Location).post(|| async move { todo("location create") }))
			.route(routes::LOGOUT, routing::get(Self::handle_get_logout))
			.route(routes::ORGANIZATION, route!(Organization).post(|| async move { todo("organization create") }))
			.route(routes::ROLE, route!(Role).post(|| async move { todo("role create") }))
			.route(
				routes::TIMESHEET,
				routing::delete(|| async move { todo("Delete method not implemented") })
					.get(
						|Extension(user): Extension<User>,
						 State(state): State<ServerState<A::Db>>,
						 Json(request): Json<request::Retrieve<MatchTimesheet>>| async move {
							let mut condition = request.into_condition();
							let code = match state
								.timesheet_permissions::<Retrieve<Timesheet>>(&user, Action::Retrieve)
								.await?
							{
								Object::Timesheet => Code::Success,

								// HACK: no if-let guards
								Object::TimesheetInDepartment if user.employee().is_some() =>
								{
									condition.job.departments &=
										MatchDepartment::from(user.employee().unwrap().department.id).into();

									Code::SuccessForPermissions
								},

								// HACK: no if-let guards
								Object::CreatedTimesheet if user.employee().is_some() =>
								{
									condition.employee.id &= user.employee().unwrap().id.into();
									Code::SuccessForPermissions
								},

								Object::TimesheetInDepartment | Object::CreatedTimesheet =>
								{
									return Ok(Response::from(Retrieve::new(
										Default::default(),
										Code::SuccessForPermissions.into(),
									)));
								},

								p => p.unreachable(),
							};

							A::Timesheet::retrieve(state.pool(), condition).await.map_or_else(
								|e| Err(Response::from(Retrieve::<Timesheet>::from(Status::from(&e)))),
								|vec| Ok(Response::from(Retrieve::new(vec, code.into()))),
							)
						},
					)
					.patch(|| async move { todo("Update method not implemented") })
					.post(|| async move { todo("timesheet create") }),
			)
			.route(
				routes::USER,
				routing::delete(|| async move { todo("Delete method not implemented") })
					.get(
						|Extension(user): Extension<User>,
						 State(state): State<ServerState<A::Db>>,
						 Json(request): Json<request::Retrieve<MatchUser>>| async move {
							let mut condition = request.into_condition();
							#[rustfmt::skip]
							let code = match state.user_permissions::<Retrieve<User>>(&user, Action::Retrieve).await?
							{
								Some(Object::User) => Code::Success,

								// HACK: no if-let guards
								Some(Object::UserInDepartment) if user.employee().is_some() =>
								{
									condition.employee = condition.employee.map(|mut m| {
										m.department.id &= user.employee().unwrap().department.id.into();
										m
									});

									Code::SuccessForPermissions
								},

								// HACK: no if-let guards
								None if user.employee().is_some() =>
								{
									let emp_id = user.employee().unwrap().id;
									condition.employee = match condition.employee
									{
										MatchOption::Any => Some(emp_id.into()).into(),
										e => e.map(|mut m| { m.id &= emp_id.into(); m }),
									};

									Code::SuccessForPermissions
								},

								Some(Object::UserInDepartment) | None =>
								{
									return Ok(Response::from(Retrieve::new(
										Default::default(),
										Code::SuccessForPermissions.into(),
									)))
								},

								Some(p) => p.unreachable(),
							};

							A::User::retrieve(state.pool(), condition).await.map_or_else(
								|e| Err(Response::from(Retrieve::<User>::from(Status::from(&e)))),
								|vec| Ok(Response::from(Retrieve::new(vec, code.into()))),
							)
						},
					)
					.patch(|| async move { todo("Update method not implemented") })
					.post(|| async move { todo("user create") }),
			)
			.route_layer(RequireAuthLayer::login())
			.route(routes::LOGIN, routing::get(Self::handle_get_login));

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

	/// The [handler](axum::Handler) for [GET](routing::get) on "/login".
	#[tracing::instrument(skip_all)]
	async fn handle_get_login(
		mut auth: AuthContext<A::Db>,
		State(state): State<ServerState<A::Db>>,
		TypedHeader(credentials): TypedHeader<Authorization<Basic>>,
	) -> impl IntoResponse
	{
		let user = match A::User::retrieve(state.pool(), MatchUser {
			username: credentials.username().to_owned().into(),
			..Default::default()
		})
		.await
		.map(|mut v| v.pop())
		{
			Ok(Some(u)) => u,
			Ok(None) => return Err(LoginResponse::invalid_credentials(None)),
			Err(e) => return Err(LoginResponse::from(e)),
		};

		PasswordHash::new(user.password()).map_or_else(
			|e| {
				tracing::error!("Failed to decode user {}'s password hash stored in database", user.username());
				Err(LoginResponse::new(
					StatusCode::INTERNAL_SERVER_ERROR,
					Status::new(Code::EncodingError, e.to_string()),
				))
			},
			|hash| {
				Argon2::default().verify_password(credentials.password().as_bytes(), &hash).map_err(|e| {
					tracing::info!("Invalid login attempt for user {}", user.username());
					LoginResponse::from(e)
				})
			},
		)?;

		// HACK: no if-let chain…
		if let Some(date) = user.password_expires()
		{
			if date < Utc::now()
			{
				tracing::info!("User {} attempted to login with expired password", user.username());
				return Err(LoginResponse::expired(date));
			}
		}

		auth.login(&user).await.map_or_else(
			|e| {
				const CODE: Code = Code::LoginError;
				tracing::error!("Failed to to log in user {}: {e}", user.username());
				Err(LoginResponse::new(CODE.into(), Status::new(CODE, e.to_string())))
			},
			|_| Ok(LoginResponse::from(Code::Success)),
		)
	}

	/// The [handler](axum::Handler) for [GET](routing::get) on "/logout".
	async fn handle_get_logout(mut auth: AuthContext<A::Db>) -> impl IntoResponse
	{
		auth.logout().await;
		LogoutResponse::from(Code::Success)
	}
}

const fn todo(msg: &'static str) -> (StatusCode, &'static str)
{
	(StatusCode::NOT_IMPLEMENTED, msg)
}

#[allow(dead_code, unused_imports, unused_macros)]
#[cfg(test)]
mod tests
{
	use core::{fmt::Debug, hash::Hash};
	use std::{collections::HashSet, sync::OnceLock};

	use axum::http::header;
	use axum_login::axum_sessions::async_session::base64;
	use axum_test_helper::{RequestBuilder, TestClient};
	use casbin::{CoreApi, Enforcer};
	use csv::WriterBuilder;
	use futures::{stream, FutureExt, StreamExt, TryFutureExt};
	use mockd::{address, company, contact, currency, internet, job, name, password, words};
	use money2::{Currency, Exchange, ExchangeRates};
	use serde::{de::DeserializeOwned, Serialize};
	use sqlx::Pool;
	use tracing_test::traced_test;
	use winvoice_match::{Match, MatchContact, MatchLocation, MatchOrganization};
	use winvoice_schema::{chrono::TimeZone, ContactKind, Invoice, Money};

	#[allow(clippy::wildcard_imports)]
	use super::*;
	use crate::{
		api::response::{Login, Logout, Version},
		lock,
		r#match::{MatchRole, MatchUser},
		utils,
	};

	const DEFAULT_SESSION_TTL: Duration = Duration::from_secs(60 * 2);
	const DEFAULT_TIMEOUT: Option<Duration> = Some(Duration::from_secs(60 * 3));

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
				let TestData { client, admin: (admin, admin_password), .. } =
					setup("rejections", DEFAULT_SESSION_TTL, DEFAULT_TIMEOUT).await?;

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
							let response = client.get(route).header(api::HEADER, version_req()).send().await;
							assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
						}

						{// assert GETs w/ wrong body are rejected
							login(&client, admin.username(), &admin_password).await;

							let response = client.get(route).header(api::HEADER, version_req()).body("").send().await;
							assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);

							logout(&client).await;
						}
					}
				})
				.await;

				Ok(())
			}
		};
	}

	/// Make a GET [`RequestBuilder`] on the given `route`.
	fn get_request_builder(client: &TestClient, route: &str) -> RequestBuilder
	{
		client.get(route).header(api::HEADER, version_req())
	}

	async fn login(client: &TestClient, username: &str, password: &str)
	{
		use pretty_assertions::assert_eq;

		let response = client
			.get(routes::LOGIN)
			.header(api::HEADER, version_req())
			.header(header::AUTHORIZATION, format!("Basic {}", base64::encode(format!("{username}:{password}"))))
			.send()
			.await;

		let expected = LoginResponse::from(Code::Success);
		assert_eq!(response.status(), expected.status());
		assert_eq!(&response.json::<Login>().await, expected.content());
	}

	async fn logout(client: &TestClient)
	{
		use pretty_assertions::assert_eq;

		let response = client.get(routes::LOGOUT).header(api::HEADER, version_req()).send().await;

		let expected = LogoutResponse::from(Code::Success);
		assert_eq!(response.status(), expected.status());
		assert_eq!(&response.json::<Logout>().await, expected.content());
	}

	#[tracing::instrument(skip(client))]
	async fn test_get_success<'ent, M, E, Iter>(
		client: &TestClient,
		route: &str,
		admin: &User,
		admin_password: &str,
		condition: M,
		entities: Iter,
		code: Option<Code>,
	) where
		E: 'ent + Clone + Debug + DeserializeOwned + Eq + Hash + PartialEq + Serialize,
		Iter: Debug + Iterator<Item = &'ent E>,
		M: Debug + Serialize,
	{
		use pretty_assertions::assert_eq;

		// HACK: `tracing` doesn't work correctly with asyn cso I have to annotate this function
		// like       this or else this function's span is skipped.
		tracing::trace!(parent: None, "\n");
		tracing::trace!("\n");

		// assert logged in user without permissions is rejected
		login(&client, admin.username(), &admin_password).await;
		let response = get_request_builder(client, route).json(&request::Retrieve::new(condition)).send().await;
		let status = response.status();

		let actual = Response::new(status, response.json::<Retrieve<E>>().await);
		let expected = Response::from(Retrieve::<E>::new(
			entities.into_iter().cloned().collect(),
			code.unwrap_or(Code::Success).into(),
		));

		assert_eq!(
			actual.content().entities().into_iter().collect::<HashSet<_>>(),
			expected.content().entities().into_iter().collect::<HashSet<_>>()
		);
		assert_eq!(actual.content().status(), expected.content().status());
		assert_eq!(actual.status(), expected.status());
		logout(&client).await;
	}

	#[tracing::instrument(skip(client))]
	async fn test_get_unauthorized<'ent, M>(client: &TestClient, route: &str, guest: &User, guest_password: &str)
	where
		M: Debug + Default + Serialize,
	{
		use pretty_assertions::assert_eq;

		// HACK: `tracing` doesn't work correctly with asyn cso I have to annotate this function
		// like       this or else this function's span is skipped.
		tracing::trace!(parent: None, "\n");
		tracing::trace!("\n");

		// assert logged in user without permissions is rejected
		login(&client, guest.username(), &guest_password).await;
		let response = get_request_builder(client, route).json(&request::Retrieve::new(M::default())).send().await;

		let actual = Response::new(response.status(), response.json::<Retrieve<()>>().await);
		let expected = Response::from(Retrieve::<()>::from(Status::from(Code::Unauthorized)));

		assert_eq!(actual.status(), expected.status());
		assert_eq!(actual.content().entities(), &[]);
		assert_eq!(actual.content().status().code(), expected.content().status().code());
		logout(&client).await;
	}

	/// Get the default version requirement for tests.
	fn version_req() -> &'static str
	{
		static VERSION_REQ: OnceLock<String> = OnceLock::new();
		VERSION_REQ.get_or_init(|| format!("={}", api::version()))
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
		async fn get() -> DynResult<()>
		{
			let TestData {
				admin: (admin, admin_password),
				client,
				grunt: (grunt, grunt_password),
				guest: (guest, guest_password),
				manager: (manager, manager_password),
				pool,
			} = setup("employee_get", DEFAULT_SESSION_TTL, DEFAULT_TIMEOUT).await?;

			let contact_ = PgContact::create(&pool, ContactKind::Email(contact::email()), words::sentence(4)).await?;

			#[rustfmt::skip]
			test_get_success(
				&client, routes::CONTACT,
				&admin, &admin_password,
				MatchContact::from(contact_.label.clone()),
				[&contact_].into_iter(), None,
			)
			.then(|_| test_get_unauthorized::<MatchContact>(&client, routes::CONTACT, &guest, &guest_password))
			.then(|_| test_get_unauthorized::<MatchContact>(&client, routes::CONTACT, &grunt, &grunt_password))
			.then(|_| test_get_unauthorized::<MatchContact>(&client, routes::CONTACT, &manager, &manager_password))
			.await;

			let department = PgDepartment::create(&pool, rand_department_name()).await?;

			#[rustfmt::skip]
			test_get_success(
				&client, routes::DEPARTMENT,
				&admin, &admin_password,
				MatchDepartment::from(department.id),
				[&department].into_iter(), None,
			)
			.then(|_| test_get_unauthorized::<MatchDepartment>(&client, routes::DEPARTMENT, &guest, &guest_password))
			.then(|_| test_get_unauthorized::<MatchDepartment>(&client, routes::DEPARTMENT, &grunt, &grunt_password))
			.then(|_| test_get_success(
				&client, routes::DEPARTMENT,
				&manager, &manager_password,
				MatchDepartment::default(),
				manager.employee().into_iter().map(|e| &e.department), Code::SuccessForPermissions.into(),
			))
			.await;

			let employee = PgEmployee::create(&pool, department.clone(), name::full(), job::title()).await?;

			#[rustfmt::skip]
			test_get_success(
				&client, routes::EMPLOYEE,
				&admin, &admin_password,
				MatchEmployee::from(employee.id),
				[&employee].into_iter(), None,
			)
			.then(|_| test_get_success(
				&client, routes::EMPLOYEE,
				&grunt, &grunt_password,
				MatchEmployee::default(),
				grunt.employee().into_iter(), Code::SuccessForPermissions.into(),
			))
			.then(|_| test_get_success(
				&client, routes::EMPLOYEE,
				&guest, &guest_password,
				MatchEmployee::default(),
				guest.employee().into_iter(), Code::SuccessForPermissions.into(),
			))
			.then(|_| test_get_success(
				&client, routes::EMPLOYEE,
				&manager, &manager_password,
				MatchEmployee::default(),
				[&grunt, &manager].into_iter().map(|e| e.employee().unwrap()), Code::SuccessForPermissions.into(),
			))
			.await;

			let location = PgLocation::create(&pool, utils::rand_currency().into(), address::country(), None).await?;

			#[rustfmt::skip]
			test_get_success(
				&client, routes::LOCATION,
				&admin, &admin_password,
				MatchLocation::from(location.id),
				[&location].into_iter(), None,
			)
			.then(|_| test_get_unauthorized::<MatchLocation>(&client, routes::LOCATION, &guest, &guest_password))
			.then(|_| test_get_unauthorized::<MatchLocation>(&client, routes::LOCATION, &grunt, &grunt_password))
			.then(|_| test_get_unauthorized::<MatchLocation>(&client, routes::LOCATION, &manager, &manager_password))
			.await;

			let organization = PgOrganization::create(&pool, location.clone(), company::company()).await?;

			#[rustfmt::skip]
			test_get_success(
				&client, routes::ORGANIZATION,
				&admin, &admin_password,
				MatchOrganization::from(organization.id),
				[&organization].into_iter(), None,
			)
			.then(|_| test_get_unauthorized::<MatchOrganization>(&client, routes::ORGANIZATION, &guest, &guest_password))
			.then(|_| test_get_unauthorized::<MatchOrganization>(&client, routes::ORGANIZATION, &grunt, &grunt_password))
			.then(|_|
				test_get_unauthorized::<MatchOrganization>(&client, routes::ORGANIZATION, &manager, &manager_password))
			.await;

			let rates = ExchangeRates::new().await?;

			let [job_, job2]: [_; 2] = {
				let mut tx = pool.begin().await?;
				let j = PgJob::create(
					&mut tx,
					organization.clone(),
					None,
					Utc::now(),
					[department.clone()].into_iter().collect(),
					Duration::new(7640, 0),
					Invoice { date: None, hourly_rate: Money::new(20_38, 2, utils::rand_currency()) },
					words::sentence(5),
					words::sentence(5),
				)
				.await?;

				let j2 = PgJob::create(
					&mut tx,
					organization.clone(),
					None,
					Utc::now(),
					manager.employee().into_iter().map(|e| e.department.clone()).collect(),
					Duration::new(7640, 0),
					Invoice { date: None, hourly_rate: Money::new(20_38, 2, utils::rand_currency()) },
					words::sentence(5),
					words::sentence(5),
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
			test_get_success(
				&client, routes::JOB,
				&admin, &admin_password,
				MatchJob::from(job_.id),
				[&job_].into_iter(), None,
			)
			.then(|_| test_get_unauthorized::<MatchJob>(&client, routes::JOB, &guest, &guest_password))
			.then(|_| test_get_unauthorized::<MatchJob>(&client, routes::JOB, &grunt, &grunt_password))
			.then(|_| test_get_success(
				&client, routes::JOB,
				&manager, &manager_password,
				MatchJob::default(),
				[&job2].into_iter(), Code::SuccessForPermissions.into(),
			))
			.await;

			let [timesheet, timesheet2, timesheet3]: [_; 3] = {
				let mut tx = pool.begin().await?;
				let t = PgTimesheet::create(
					&mut tx,
					employee.clone(),
					Default::default(),
					job_.clone(),
					Utc.with_ymd_and_hms(2022, 06, 08, 15, 27, 00).unwrap(),
					Utc.with_ymd_and_hms(2022, 06, 09, 07, 00, 00).latest(),
					words::sentence(5),
				)
				.await?;

				let t2 = PgTimesheet::create(
					&mut tx,
					grunt.employee().unwrap().clone(),
					Default::default(),
					job2.clone(),
					Utc.with_ymd_and_hms(2022, 06, 08, 15, 27, 00).unwrap(),
					Utc.with_ymd_and_hms(2022, 06, 09, 07, 00, 00).latest(),
					words::sentence(5),
				)
				.await?;

				let t3 = PgTimesheet::create(
					&mut tx,
					manager.employee().unwrap().clone(),
					Default::default(),
					job2.clone(),
					Utc.with_ymd_and_hms(2022, 06, 08, 15, 27, 00).unwrap(),
					Utc.with_ymd_and_hms(2022, 06, 09, 07, 00, 00).latest(),
					words::sentence(5),
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
			test_get_success(
				&client, routes::TIMESHEET,
				&admin, &admin_password,
				MatchTimesheet::from(timesheet.id),
				[&timesheet].into_iter(), None,
			)
			.then(|_| test_get_unauthorized::<MatchTimesheet>(&client, routes::TIMESHEET, &guest, &guest_password))
			.then(|_| test_get_success(
				&client, routes::TIMESHEET,
				&grunt, &grunt_password,
				MatchTimesheet::default(),
				[&timesheet2].into_iter(), Code::SuccessForPermissions.into(),
			))
			.then(|_| test_get_success(
				&client, routes::TIMESHEET,
				&manager, &manager_password,
				MatchTimesheet::default(),
				[&timesheet2, &timesheet3].into_iter(), Code::SuccessForPermissions.into(),
			))
			.await;

			let expenses = {
				let mut x = Vec::with_capacity(2 * 3);
				for t in [&timesheet, &timesheet2, &timesheet3]
				{
					#[rustfmt::skip]
					PgExpenses::create(
						&pool,
						vec![
							(
								words::word(),
								Money::new(20_00, 2, utils::rand_currency()),
								words::sentence(5),
							),
							(
								words::word(),
								Money::new(737_00, 2, utils::rand_currency()),
								words::sentence(5),
							),
						],
						t.id,
					)
					.await
					.map(|mut v| x.append(&mut v))?;
				}

				x.exchange(Default::default(), &rates)
			};

			#[rustfmt::skip]
			test_get_success(
				&client, routes::EXPENSE,
				&admin, &admin_password,
				MatchExpense::from(Match::Or(expenses.iter().map(|x| x.id.into()).collect())),
				expenses.iter(), None,
			)
			.then(|_| test_get_unauthorized::<MatchExpense>(&client, routes::EXPENSE, &guest, &guest_password))
			.then(|_| test_get_success(
				&client, routes::EXPENSE,
				&grunt, &grunt_password,
				MatchExpense::default(),
				expenses.iter().filter(|x| x.timesheet_id == timesheet2.id), Code::SuccessForPermissions.into(),
			))
			.then(|_| test_get_success(
				&client, routes::EXPENSE,
				&manager, &manager_password,
				MatchExpense::default(),
				expenses.iter().filter(|x| x.timesheet_id == timesheet2.id || x.timesheet_id == timesheet3.id),
				Code::SuccessForPermissions.into(),
			))
			.await;

			let users = serde_json::to_string(&[&admin, &guest, &grunt, &manager])
				.and_then(|json| serde_json::from_str::<[User; 4]>(&json))?;

			let roles = users.iter().map(|u| u.role().clone()).collect::<Vec<_>>();
			test_get_success(
				&client,
				routes::ROLE,
				&admin,
				&admin_password,
				MatchRole::from(Match::Or(roles.iter().map(|r| r.id().into()).collect())),
				roles.iter(),
				None,
			)
			.then(|_| test_get_unauthorized::<MatchRole>(&client, routes::ROLE, &guest, &guest_password))
			.then(|_| test_get_unauthorized::<MatchRole>(&client, routes::ROLE, &grunt, &grunt_password))
			.then(|_| test_get_unauthorized::<MatchRole>(&client, routes::ROLE, &manager, &manager_password))
			.await;

			#[rustfmt::skip]
			test_get_success(
				&client, routes::USER,
				&admin, &admin_password,
				MatchUser::from(Match::Or(users.iter().map(|u| u.id().into()).collect())),
				users.iter(), None,
			)
			.then(|_| test_get_success(
				&client, routes::USER,
				&grunt, &grunt_password,
				MatchUser::default(),
				users.iter().filter(|u| u.id() == grunt.id()), Code::SuccessForPermissions.into(),
			))
			.then(|_| test_get_success(
				&client, routes::USER,
				&guest, &guest_password,
				MatchUser::default(),
				users.iter().filter(|u| u.id() == guest.id()), Code::SuccessForPermissions.into(),
			))
			// .then(|_| test_get_success(
			// 	&client, routes::USER,
			// 	&manager, &manager_password,
			// 	MatchUser::default(),
			// 	[&grunt, &manager].into_iter(), Code::SuccessForPermissions.into(),
			// ))
			.await;

			PgUser::delete(&pool, users.iter()).await?;
			futures::try_join!(PgRole::delete(&pool, roles.iter()), PgJob::delete(&pool, [&job_, &job2].into_iter()))?;

			PgOrganization::delete(&pool, [organization].iter()).await?;
			futures::try_join!(
				PgContact::delete(&pool, [&contact_].into_iter()),
				PgEmployee::delete(&pool, [&employee].into_iter()),
				PgLocation::delete(&pool, [&location].into_iter()),
			)?;

			Ok(())
		}
	}
}
