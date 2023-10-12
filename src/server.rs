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
						header::ACCEPT,
						header::ACCEPT_CHARSET,
						header::ACCEPT_ENCODING,
						header::ACCEPT_LANGUAGE,
						header::ACCEPT_RANGES,
						header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
						header::ACCESS_CONTROL_ALLOW_HEADERS,
						header::ACCESS_CONTROL_ALLOW_METHODS,
						header::ACCESS_CONTROL_ALLOW_ORIGIN,
						header::ACCESS_CONTROL_EXPOSE_HEADERS,
						header::ACCESS_CONTROL_MAX_AGE,
						header::ACCESS_CONTROL_REQUEST_HEADERS,
						header::ACCESS_CONTROL_REQUEST_METHOD,
						header::AGE,
						header::ALLOW,
						header::ALT_SVC,
						header::AUTHORIZATION,
						header::CACHE_CONTROL,
						header::CACHE_STATUS,
						header::CDN_CACHE_CONTROL,
						header::CONNECTION,
						header::CONTENT_DISPOSITION,
						header::CONTENT_ENCODING,
						header::CONTENT_LANGUAGE,
						header::CONTENT_LENGTH,
						header::CONTENT_LOCATION,
						header::CONTENT_RANGE,
						header::CONTENT_SECURITY_POLICY,
						header::CONTENT_SECURITY_POLICY_REPORT_ONLY,
						header::CONTENT_TYPE,
						header::COOKIE,
						header::DATE,
						header::DNT,
						header::ETAG,
						header::EXPECT,
						header::EXPIRES,
						header::FORWARDED,
						header::FROM,
						header::HOST,
						header::IF_MATCH,
						header::IF_MODIFIED_SINCE,
						header::IF_NONE_MATCH,
						header::IF_RANGE,
						header::IF_UNMODIFIED_SINCE,
						header::LAST_MODIFIED,
						header::LINK,
						header::LOCATION,
						header::MAX_FORWARDS,
						header::ORIGIN,
						header::PRAGMA,
						header::PROXY_AUTHENTICATE,
						header::PROXY_AUTHORIZATION,
						header::PUBLIC_KEY_PINS,
						header::PUBLIC_KEY_PINS_REPORT_ONLY,
						header::RANGE,
						header::REFERER,
						header::REFERRER_POLICY,
						header::REFRESH,
						header::RETRY_AFTER,
						header::SEC_WEBSOCKET_ACCEPT,
						header::SEC_WEBSOCKET_EXTENSIONS,
						header::SEC_WEBSOCKET_KEY,
						header::SEC_WEBSOCKET_PROTOCOL,
						header::SEC_WEBSOCKET_VERSION,
						header::SERVER,
						header::SET_COOKIE,
						header::STRICT_TRANSPORT_SECURITY,
						header::TE,
						header::TRAILER,
						header::TRANSFER_ENCODING,
						header::UPGRADE,
						header::UPGRADE_INSECURE_REQUESTS,
						header::USER_AGENT,
						header::VARY,
						header::VIA,
						header::WARNING,
						header::WWW_AUTHENTICATE,
						header::X_CONTENT_TYPE_OPTIONS,
						header::X_DNS_PREFETCH_CONTROL,
						header::X_FRAME_OPTIONS,
						header::X_XSS_PROTECTION,
					])
					.allow_methods([Method::DELETE, Method::GET, Method::PATCH, Method::PUT])
					.allow_origin(cors_allow_origin)
					.allow_private_network(true),
			)
			.layer(TraceLayer::new_for_http())
			.with_state(state))
	}
}
