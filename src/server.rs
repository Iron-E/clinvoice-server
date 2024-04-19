//! The `server` module functions to spawn an [`axum_server`] which communicates over TLS.

mod auth;
mod db_session_store;
mod handler;
mod response;
mod state;
#[cfg(test)]
mod test_client_ext;
#[cfg(test)]
mod tests;

use core::{fmt::Display, marker::PhantomData, time::Duration};
use std::net::SocketAddr;

use auth::{DbUserStore, InitializableWithAuthorization, RequireAuthLayer, UserStore};
use axum::{
	error_handling::HandleErrorLayer,
	http::{
		header::{self, HeaderMap, HeaderName},
		HeaderValue,
		Method,
		Request,
		StatusCode,
	},
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
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};
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
		cors_allow_origin: Vec<HeaderValue>,
		state: ServerState<A::Db>,
		session_ttl: Duration,
		timeout: Option<Duration>,
	) -> DynResult<()>
	{
		let router = Self::router(cookie_domain, cookie_secret, cors_allow_origin, state, session_ttl, timeout).await?;
		axum_server::bind_rustls(self.address, self.tls).serve(router.into_make_service()).await?;
		Ok(())
	}

	/// Create the [`Router`] that will be used by the [`Server`].
	async fn router(
		cookie_domain: Option<String>,
		cookie_secret: Vec<u8>,
		cors_allow_origin: Vec<HeaderValue>,
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
			.route(routes::EXPORT, handler.export())
			.route(routes::JOB, handler.job())
			.route(routes::LOCATION, handler.location())
			.route(routes::LOGOUT, handler.logout())
			.route(routes::ORGANIZATION, handler.organization())
			.route(routes::ROLE, handler.role())
			.route(routes::TIMESHEET, handler.timesheet())
			.route(routes::USER, handler.user())
			.route(routes::WHO_AM_I, handler.who_am_i())
			.route_layer(RequireAuthLayer::login())
			.route(routes::HEALTHY, handler.healthy())
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
			.layer(
				CorsLayer::new()
					.allow_credentials(true)
					.allow_headers([
						HeaderName::from_static(api::HEADER),
						HeaderName::from_static("sec-fetch-dest"),
						HeaderName::from_static("sec-fetch-mode"),
						HeaderName::from_static("sec-fetch-site"),
						header::ACCEPT,
						header::ACCEPT_ENCODING,
						header::ACCEPT_LANGUAGE,
						header::ACCESS_CONTROL_REQUEST_HEADERS,
						header::ACCESS_CONTROL_REQUEST_METHOD,
						header::AUTHORIZATION,
						header::CONNECTION,
						header::CONTENT_LENGTH,
						header::CONTENT_TYPE,
						header::COOKIE,
						header::DNT,
						header::HOST,
						header::ORIGIN,
						header::REFERER,
						header::TE,
						header::USER_AGENT,
					])
					.allow_methods([Method::DELETE, Method::POST, Method::PATCH, Method::PUT])
					.allow_origin(cors_allow_origin)
					.allow_private_network(true),
			)
			.layer(TraceLayer::new_for_http())
			.with_state(state))
	}
}
