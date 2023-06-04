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
pub struct Server<Db>
{
	/// The [`SocketAddr`] that self server is bound to.
	address: SocketAddr,

	phantom: PhantomData<Db>,

	/// The TLS configuration.
	tls: RustlsConfig,
}

impl<Db> Server<Db>
where
	Db: Database,
	Db::Connection: core::fmt::Debug,
	<Db::Connection as Connection>::Options: Clone,
	DbSessionStore<Db>: Initializable<Db = Db> + SessionStore,
	SqlxStore<Pool<Db>, User>: UserStore<Id, (), User = User>,
	for<'connection> &'connection mut Db::Connection: Executor<'connection, Database = Db>,
	for<'connection> &'connection mut Transaction<'connection, Db>:
		Executor<'connection, Database = Db>,
{
	/// Create a new [`Server`]
	pub const fn new(address: SocketAddr, tls: RustlsConfig) -> Self
	{
		Self { address, phantom: PhantomData, tls }
	}

	/// Create an [`Router`] based on the `connect_options`.
	///
	/// Operations `timeout`, if specified.
	pub async fn serve<A>(
		self,
		cookie_domain: Option<String>,
		cookie_secret: Vec<u8>,
		state: ServerState<Db>,
		session_ttl: Duration,
		timeout: Option<Duration>,
	) -> DynResult<()>
	where
		A: Adapter<Db = Db> + InitializableWithAuthorization,
	{
		let router =
			Self::router::<A>(cookie_domain, cookie_secret, state, session_ttl, timeout).await?;
		axum_server::bind_rustls(self.address, self.tls).serve(router.into_make_service()).await?;
		Ok(())
	}

	/// Create a new [`MethodRouter`] with [`delete`](routing::delete) and [`patch`](routing::patch)
	/// preconfigured, since those are common among all Winvoice entities.
	fn route<TEntity>() -> MethodRouter<ServerState<Db>>
	where
		TEntity: Deletable<Db = Db> + Retrievable<Db = Db> + Updatable<Db = Db>,
	{
		routing::delete(|| async { todo("Delete method not implemented") })
			.patch(|| async { todo("Update method not implemented") })
	}

	/// Create the [`Router`] that will be used by the [`Server`].
	async fn router<A>(
		cookie_domain: Option<String>,
		cookie_secret: Vec<u8>,
		state: ServerState<Db>,
		session_ttl: Duration,
		timeout: Option<Duration>,
	) -> DynResult<Router>
	where
		A: Adapter<Db = Db> + InitializableWithAuthorization,
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

		// async fn login_handler(mut auth: AuthContext)
		// {
		// 	let pool = SqlitePoolOptions::new().connect("sqlite/user_store.db").await.unwrap();
		// 	let mut conn = pool.acquire().await.unwrap();
		// 	let user: User = sqlx::query_as("select * from users where id = 1")
		// 		.fetch_one(&mut conn)
		// 		.await
		// 		.unwrap();
		// 	auth.login(&user).await.unwrap();
		// }

		// async fn logout_handler(mut auth: AuthContext)
		// {
		// 	dbg!("Logging out user: {}", &auth.current_user);
		// 	auth.logout().await;
		// }

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
			.route(
				"/login",
				routing::get(
					|mut auth: AuthContext<Db>,
					 State(state): State<ServerState<Db>>,
					 TypedHeader(credentials): TypedHeader<Authorization<Basic>>| async move {
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
							.and_then(|hash| {
								Argon2::default().verify_password(user.password().as_bytes(), &hash)
							})
							.map_err(LoginResponse::from)?;

						auth.login(&user).await.map(|_| LoginResponse::success()).map_err(|e| {
							let code = Code::LoginError;
							LoginResponse::new(code.into(), Status::new(code, e.to_string()))
						})
					},
				),
			)
			.route("/logout", routing::put(|| async { todo("logout") }))
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
}

const fn todo(msg: &'static str) -> (StatusCode, &'static str)
{
	(StatusCode::NOT_IMPLEMENTED, msg)
}
