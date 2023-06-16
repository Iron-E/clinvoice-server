//! The `server` module functions to spawn an [`axum_server`] which communicates over TLS.

mod auth;
mod db_session_store;
mod response;
mod state;

use core::{fmt::Display, marker::PhantomData, time::Duration};
use std::net::SocketAddr;

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
use winvoice_schema::chrono::Utc;

use crate::{
	api::{self, request, response::Retrieve, routes, Code, Status},
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
					state.has_permission(&user, Object::$Entity, Action::Retrieve).await.map_err(
						|status| {
							Response::from(Retrieve::<<A::$Entity as Retrievable>::Entity>::from(
								status,
							))
						},
					)?;

					let condition = request.into_condition();
					A::$Entity::retrieve(state.pool(), condition).await.map_or_else(
						|e| {
							Err(Response::from(
								Retrieve::<<A::$Entity as Retrievable>::Entity>::from(
									Status::from(&e),
								),
							))
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
	for<'connection> &'connection mut <A::Db as Database>::Connection:
		Executor<'connection, Database = A::Db>,
	for<'connection> &'connection mut Transaction<'connection, A::Db>:
		Executor<'connection, Database = A::Db>,
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
		let router =
			Self::router(cookie_domain, cookie_secret, state, session_ttl, timeout).await?;
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
								match req.matches(api::version())
								{
									false => Err(VersionResponse::mismatch()),
									true => Ok(()),
								}
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
				routes::EMPLOYEE,
				route!(Employee).post(|| async move { todo("employee create") }),
			)
			.route(routes::EXPENSE, route!(Expenses).post(|| async move { todo("expense create") }))
			.route(routes::JOB, route!(Job).post(|| async move { todo("job create") }))
			.route(
				routes::LOCATION,
				route!(Location).post(|| async move { todo("location create") }),
			)
			.route(routes::LOGOUT, routing::get(Self::handle_get_logout))
			.route(
				routes::ORGANIZATION,
				route!(Organization).post(|| async move { todo("organization create") }),
			)
			.route(routes::ROLE, route!(Role).post(|| async move { todo("role create") }))
			.route(
				routes::TIMESHEET,
				route!(Timesheet).post(|| async move { todo("timesheet create") }),
			)
			.route(routes::USER, route!(User).post(|| async move { todo("user create") }))
			.route_layer(RequireAuthLayer::login())
			.route(routes::LOGIN, routing::get(Self::handle_get_login));

		if let Some(t) = timeout
		{
			router = router.layer(
				ServiceBuilder::new()
					.layer(HandleErrorLayer::new(|err: BoxError| async move {
						match err.is::<timeout::error::Elapsed>()
						{
							#[rustfmt::skip]
							true => (StatusCode::REQUEST_TIMEOUT, "Request took too long".to_owned()),
							false => (
								StatusCode::INTERNAL_SERVER_ERROR,
								format!("Unhandled internal error: {err}"),
							),
						}
					}))
					.timeout(t),
			);
		}

		Ok(router
			.layer(AuthLayer::new(
				SqlxStore::<_, User>::new(state.pool().clone()).with_query({
					let mut query = QueryBuilder::<A::Db>::from(A::User::default());
					query
						.push(sql::WHERE)
						.push(UserColumns::default().default_scope().id)
						.push(" = $1");
					query.into_sql()
				}),
				&cookie_secret,
			))
			.layer({
				let mut layer = SessionLayer::new(session_store, &cookie_secret)
					.with_session_ttl(session_ttl.into());

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
				tracing::error!(
					"Failed to decode user {}'s password hash stored in database",
					user.username()
				);
				Err(LoginResponse::new(
					StatusCode::INTERNAL_SERVER_ERROR,
					Status::new(Code::EncodingError, e.to_string()),
				))
			},
			|hash| {
				Argon2::default().verify_password(credentials.password().as_bytes(), &hash).map_err(
					|e| {
						tracing::info!("Invalid login attempt for user {}", user.username());
						LoginResponse::from(e)
					},
				)
			},
		)?;

		if user.password_expires().map_or(false, |date| date < Utc::now())
		{
			tracing::info!("User {} attempted to login with expired password", user.username());
			return Err(LoginResponse::expired(user.password_expires().unwrap()));
		}

		auth.login(&user).await.map(|_| LoginResponse::from(Code::Success)).map_err(|e| {
			const CODE: Code = Code::LoginError;
			tracing::error!("Failed to to log in user {}: {e}", user.username());
			LoginResponse::new(CODE.into(), Status::new(CODE, e.to_string()))
		})
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

#[cfg(test)]
mod tests
{
	use core::fmt::Debug;
	use std::sync::OnceLock;

	use axum::http::header;
	use axum_login::axum_sessions::async_session::base64;
	use axum_test_helper::{RequestBuilder, TestClient};
	use casbin::{CoreApi, Enforcer};
	use futures::{stream, StreamExt, TryFutureExt};
	use mockd::{address, company, contact, currency, internet, job, name, password, words};
	use money2::Currency;
	use serde::{de::DeserializeOwned, Serialize};
	use sqlx::Pool;
	use tracing_test::traced_test;
	use winvoice_match::{MatchContact, MatchEmployee, MatchJob, MatchLocation, MatchOrganization};
	use winvoice_schema::ContactKind;

	#[allow(clippy::wildcard_imports)]
	use super::*;
	use crate::{
		api::response::{Login, Logout, Version},
		lock,
		utils,
	};

	const DEFAULT_SESSION_TTL: Duration = Duration::from_secs(60 * 2);
	const DEFAULT_TIMEOUT: Option<Duration> = Some(Duration::from_secs(60 * 3));

	macro_rules! fn_setup {
		($Adapter:ty, $Db:ty, $connect:path) => {
			/// Setup for the tests.
			///
			/// # Returns
			///
			/// * `(client, pool, admin, admin_password, guest, guest_password)`
			async fn setup(
				test: &str,
				session_ttl: Duration,
				time_out: Option<Duration>,
			) -> DynResult<(TestClient, Pool<$Db>, User, String, User, String)>
			{
				let admin_role_name = name::full();
				let policy = format!(
					"p, {admin_role_name}, {contact}, {create}
p, {admin_role_name}, {contact}, {delete}
p, {admin_role_name}, {contact}, {retrieve}
p, {admin_role_name}, {contact}, {update}
p, {admin_role_name}, {employee}, {create}
p, {admin_role_name}, {employee}, {delete}
p, {admin_role_name}, {employee}, {retrieve}
p, {admin_role_name}, {employee}, {update}
p, {admin_role_name}, {expenses}, {create}
p, {admin_role_name}, {expenses}, {delete}
p, {admin_role_name}, {expenses}, {retrieve}
p, {admin_role_name}, {expenses}, {update}
p, {admin_role_name}, {job}, {create}
p, {admin_role_name}, {job}, {delete}
p, {admin_role_name}, {job}, {retrieve}
p, {admin_role_name}, {job}, {update}
p, {admin_role_name}, {location}, {create}
p, {admin_role_name}, {location}, {delete}
p, {admin_role_name}, {location}, {retrieve}
p, {admin_role_name}, {location}, {update}
p, {admin_role_name}, {organization}, {create}
p, {admin_role_name}, {organization}, {delete}
p, {admin_role_name}, {organization}, {retrieve}
p, {admin_role_name}, {organization}, {update}
p, {admin_role_name}, {role}, {create}
p, {admin_role_name}, {role}, {delete}
p, {admin_role_name}, {role}, {retrieve}
p, {admin_role_name}, {role}, {update}
p, {admin_role_name}, {timesheet}, {create}
p, {admin_role_name}, {timesheet}, {delete}
p, {admin_role_name}, {timesheet}, {retrieve}
p, {admin_role_name}, {timesheet}, {update}
p, {admin_role_name}, {user}, {create}
p, {admin_role_name}, {user}, {delete}
p, {admin_role_name}, {user}, {retrieve}
p, {admin_role_name}, {user}, {update}
",
					create = Action::Create,
					delete = Action::Delete,
					retrieve = Action::Retrieve,
					update = Action::Update,
					contact = Object::Contact,
					employee = Object::Employee,
					expenses = Object::Expenses,
					job = Object::Job,
					location = Object::Location,
					organization = Object::Organization,
					role = Object::Role,
					timesheet = Object::Timesheet,
					user = Object::User,
				);

				#[rustfmt::skip]
				let (model_path, policy_path) = utils::init_model_and_policy_files(
					&format!("server::{}::{test}", stringify!($Adapter)),
					utils::Model::Rbac.to_string(),
					policy,
				)
				.await
				.map(|(m, p)| (
						utils::leak_string(m.to_string_lossy().into()),
						utils::leak_string(p.to_string_lossy().into()),
				))?;

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
				let guest_password = password::generate(true, true, true, 8);

				#[rustfmt::skip]
				let (admin, guest) = futures::try_join!(
					<$Adapter as winvoice_adapter::schema::Adapter>::Employee::create(&pool,
						"Geoff".into(), "Hired".into(), "Manager".into()
					)
					.and_then(|employee| <$Adapter as Adapter>::Role::create(&pool,
						admin_role_name, Duration::from_secs(60).into(),
					)
					.and_then(|role| <$Adapter as Adapter>::User::create(&pool,
						employee.into(), admin_password.to_owned(), role, internet::username(),
					))),

					<$Adapter as winvoice_adapter::schema::Adapter>::Employee::create(&pool,
						"Jiff".into(), "Suspended".into(), "CEO".into()
					)
					.and_then(|employee| <$Adapter as Adapter>::Role::create(&pool,
						words::sentence(4), Duration::from_secs(60).into(),
					)
					.and_then(|role| <$Adapter as Adapter>::User::create(&pool,
						employee.into(), guest_password.to_owned(), role, internet::username(),
					))),
				)?;

				Ok((TestClient::new(server), pool, admin, admin_password, guest, guest_password))
			}

			#[tokio::test]
			#[traced_test]
			async fn rejections() -> DynResult<()>
			{
				let (client, _, admin, admin_password, ..) =
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

	async fn login(client: &TestClient, username: &str, password: &str)
	{
		let response = client
			.get(routes::LOGIN)
			.header(api::HEADER, version_req())
			.header(
				header::AUTHORIZATION,
				format!("Basic {}", base64::encode(format!("{username}:{password}"))),
			)
			.send()
			.await;

		let expected = LoginResponse::from(Code::Success);
		assert_eq!(response.status(), expected.status());
		assert_eq!(&response.json::<Login>().await, expected.content());
	}

	async fn logout(client: &TestClient)
	{
		let response = client.get(routes::LOGOUT).header(api::HEADER, version_req()).send().await;

		let expected = LogoutResponse::from(Code::Success);
		assert_eq!(response.status(), expected.status());
		assert_eq!(&response.json::<Logout>().await, expected.content());
	}

	#[tracing::instrument(skip(client, admin, admin_password, guest, guest_password))]
	async fn test_get<E, M>(
		client: &TestClient,
		route: &str,
		admin: &User,
		admin_password: &str,
		guest: &User,
		guest_password: &str,
		entity: &E,
		condition: M,
	) where
		E: Clone + Debug + DeserializeOwned + PartialEq + Serialize,
		M: Default + Debug + Serialize,
	{
		// HACK: `tracing` doesn't work correctly with asyn cso I have to annotate this function
		// like       this or else this function's span is skipped.
		tracing::trace!("");

		let get_client =
			|| -> RequestBuilder { client.get(route).header(api::HEADER, version_req()) };

		{
			// assert logged in user without permissions is rejected
			login(&client, guest.username(), &guest_password).await;
			let response = get_client().json(&request::Retrieve::new(M::default())).send().await;

			let actual = Response::new(response.status(), response.json::<Retrieve<E>>().await);
			let expected =
				Response::from(Retrieve::<E>::from(Status::new(Code::Unauthorized, "".into())));

			assert_eq!(actual.status(), expected.status());
			assert_eq!(actual.content().entities(), &[]);
			assert_eq!(actual.content().status().code(), expected.content().status().code());
			logout(&client).await;
		}

		{
			// assert logged in user without permissions is rejected
			login(&client, admin.username(), &admin_password).await;
			let response = get_client().json(&request::Retrieve::new(condition)).send().await;

			let actual = Response::new(response.status(), response.json::<Retrieve<E>>().await);
			let expected = Response::from(Retrieve::<E>::new(
				[entity].into_iter().cloned().collect(),
				Code::Success.into(),
			));

			assert_eq!(actual.content(), expected.content());
			assert_eq!(actual.status(), expected.status());
			logout(&client).await;
		}
	}

	/// Get the default version requirement for tests.
	fn version_req() -> &'static str
	{
		static VERSION_REQ: OnceLock<String> = OnceLock::new();
		VERSION_REQ.get_or_init(|| format!("={}", api::version()))
	}

	#[cfg(feature = "postgres")]
	mod postgres
	{
		use pretty_assertions::assert_eq;
		use sqlx::Postgres;
		use winvoice_adapter_postgres::{
			schema::{PgContact, PgEmployee, PgJob, PgLocation, PgOrganization},
			PgSchema,
		};

		#[allow(clippy::wildcard_imports)]
		use super::*;
		use crate::schema::postgres::{PgRole, PgUser};

		fn_setup!(PgSchema, Postgres, utils::connect_pg);

		#[tokio::test]
		#[traced_test]
		async fn get() -> DynResult<()>
		{
			let (client, pool, admin, admin_password, guest, guest_password) =
				setup("employee_get", DEFAULT_SESSION_TTL, DEFAULT_TIMEOUT).await?;

			let contact =
				PgContact::create(&pool, ContactKind::Email(contact::email()), words::sentence(4))
					.await?;
			test_get(
				&client,
				routes::CONTACT,
				&admin,
				&admin_password,
				&guest,
				&guest_password,
				&contact,
				MatchContact::from(contact.label.clone()),
			)
			.await;

			let employee =
				PgEmployee::create(&pool, name::full(), job::descriptor(), job::title()).await?;
			test_get(
				&client,
				routes::EMPLOYEE,
				&admin,
				&admin_password,
				&guest,
				&guest_password,
				&employee,
				MatchEmployee::from(employee.id),
			)
			.await;

			let c = loop {
				match currency::short().parse::<Currency>() {
					Ok(c) => break c,
					Err(_) => (),
				}
			};

			let location = PgLocation::create(&pool, c.into(), address::country(), None).await?;
			test_get(
				&client,
				routes::LOCATION,
				&admin,
				&admin_password,
				&guest,
				&guest_password,
				&location,
				MatchLocation::from(location.id),
			)
			.await;

			let organization =
				PgOrganization::create(&pool, location.clone(), company::company()).await?;
			test_get(
				&client,
				routes::ORGANIZATION,
				&admin,
				&admin_password,
				&guest,
				&guest_password,
				&organization,
				MatchOrganization::from(organization.id),
			)
			.await;

			// let job =
			// 	PgJob::create(&pool, organization.clone(), ).await?;
			// test_get(
			// 	&client,
			// 	routes::JOB,
			// 	&admin,
			// 	&admin_password,
			// 	&guest,
			// 	&guest_password,
			// 	&job,
			// 	MatchJob::from(job.id),
			// )
			// .await;

			// PgJob::delete(&pool, [job].iter()).await?;
			PgOrganization::delete(&pool, [organization].iter()).await?;
			futures::try_join!(
				PgContact::delete(&pool, [&contact].into_iter()),
				PgEmployee::delete(&pool, [&employee].into_iter()),
				PgLocation::delete(&pool, [&location].into_iter()),
			)?;

			todo!("job");
			todo!("timesheet");
			todo!("expenses");

			todo!("role");
			todo!("user");

			Ok(())
		}
	}
}
