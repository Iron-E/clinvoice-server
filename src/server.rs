//! The `server` module functions to spawn an [`axum_server`] which communicates over TLS.

mod auth;
mod db_session_store;
mod response;
mod state;

use core::{marker::PhantomData, time::Duration};
use std::net::SocketAddr;

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use auth::{AuthContext, InitializableWithAuthorization};
use axum::{
	error_handling::HandleErrorLayer,
	extract::State,
	headers::{authorization::Basic, Authorization},
	http::StatusCode,
	response::IntoResponse,
	routing::{self, MethodRouter},
	BoxError,
	Router,
	TypedHeader,
};
use axum_login::{
	axum_sessions::{async_session::SessionStore, SessionLayer},
	AuthLayer,
	SqlxStore,
	UserStore,
};
use axum_server::tls_rustls::RustlsConfig;
use db_session_store::DbSessionStore;
pub use response::{LoginResponse, LogoutResponse, Response};
use sqlx::{Connection, Database, Executor, Pool, Transaction};
pub use state::ServerState;
use tower::{timeout, ServiceBuilder};
use tower_http::{compression::CompressionLayer, trace::TraceLayer};
use winvoice_adapter::{Deletable, Initializable, Retrievable, Updatable};
use winvoice_schema::Id;

use crate::{
	api::{
		r#match::MatchUser,
		schema::{Adapter, User},
		Code,
		Status,
	},
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
	DbSessionStore<A::Db>: Initializable<Db = A::Db> + SessionStore,
	SqlxStore<Pool<A::Db>, User>: UserStore<Id, (), User = User>,
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
	where
		A: InitializableWithAuthorization,
	{
		let router =
			Self::router(cookie_domain, cookie_secret, state, session_ttl, timeout).await?;
		axum_server::bind_rustls(self.address, self.tls).serve(router.into_make_service()).await?;
		Ok(())
	}

	/// Create a new [`MethodRouter`] with [`delete`](routing::delete) and [`patch`](routing::patch)
	/// preconfigured, since those are common among all Winvoice entities.
	fn route<TEntity>() -> MethodRouter<ServerState<A::Db>>
	where
		TEntity: Deletable<Db = A::Db> + Retrievable<Db = A::Db> + Updatable<Db = A::Db>,
	{
		routing::delete(|| async { todo("Delete method not implemented") })
			.patch(|| async { todo("Update method not implemented") })
	}

	/// Create the [`Router`] that will be used by the [`Server`].
	async fn router(
		cookie_domain: Option<String>,
		cookie_secret: Vec<u8>,
		state: ServerState<A::Db>,
		session_ttl: Duration,
		timeout: Option<Duration>,
	) -> DynResult<Router>
	where
		A: InitializableWithAuthorization,
	{
		futures::try_join!(A::init_with_auth(state.pool()), DbSessionStore::init(state.pool()))?;

		let mut router = Router::new();
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
			.layer(CompressionLayer::new())
			.layer(AuthLayer::new(SqlxStore::<_, User>::new(state.pool().clone()), &cookie_secret))
			.layer({
				let mut layer =
					SessionLayer::new(DbSessionStore::new(state.pool().clone()), &cookie_secret)
						.with_session_ttl(session_ttl.into());

				if let Some(s) = cookie_domain
				{
					layer = layer.with_cookie_domain(s);
				}

				layer
			})
			.layer(TraceLayer::new_for_http())
			.route("/login", routing::get(Self::handle_get_login))
			.route("/logout", routing::get(Self::handle_get_logout))
			.route(
				"/contact",
				Self::route::<A::Contact>()
					.get(|| async { todo("contact retrieve") })
					.post(|| async { todo("contact create") }),
			)
			.route(
				"/employee",
				Self::route::<A::Employee>()
					.get(|| async { todo("employee retrieve") })
					.post(|| async { todo("employee create") }),
			)
			.route(
				"/expense",
				Self::route::<A::Expenses>()
					.get(|| async { todo("expense retrieve") })
					.post(|| async { todo("expense create") }),
			)
			.route(
				"/job",
				Self::route::<A::Job>()
					.get(|| async { todo("job retrieve") })
					.post(|| async { todo("job create") }),
			)
			.route(
				"/location",
				Self::route::<A::Location>()
					.get(|| async { todo("location retrieve") })
					.post(|| async { todo("location create") }),
			)
			.route(
				"/organization",
				Self::route::<A::Organization>()
					.get(|| async { todo("organization retrieve") })
					.post(|| async { todo("organization create") }),
			)
			.route(
				"/role",
				Self::route::<A::Role>()
					.get(|| async { todo("role retrieve") })
					.post(|| async { todo("role create") }),
			)
			.route(
				"/timesheet",
				Self::route::<A::Timesheet>()
					.get(|| async { todo("timesheet retrieve") })
					.post(|| async { todo("timesheet create") }),
			)
			.route(
				"/user",
				Self::route::<A::User>()
					.get(|| async { todo("user retrieve") })
					.post(|| async { todo("user create") }),
			)
			.with_state(state))
	}

	/// The [handler](axum::Handler) for [GET](routing::get) on "/login".
	async fn handle_get_login(
		mut auth: AuthContext<A::Db>,
		State(state): State<ServerState<A::Db>>,
		TypedHeader(credentials): TypedHeader<Authorization<Basic>>,
	) -> Result<LoginResponse, LoginResponse>
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

		PasswordHash::new(user.password())
			.and_then(|hash| Argon2::default().verify_password(user.password().as_bytes(), &hash))
			.map_err(LoginResponse::from)?;

		auth.login(&user).await.map(|_| LoginResponse::success()).map_err(|e| {
			let code = Code::LoginError;
			LoginResponse::new(code.into(), Status::new(code, e.to_string()))
		})
	}

	/// The [handler](axum::Handler) for [GET](routing::get) on "/logout".
	async fn handle_get_logout(mut auth: AuthContext<A::Db>) -> impl IntoResponse
	{
		auth.logout().await;
		LogoutResponse::success()
	}
}

const fn todo(msg: &'static str) -> (StatusCode, &'static str)
{
	(StatusCode::NOT_IMPLEMENTED, msg)
}
