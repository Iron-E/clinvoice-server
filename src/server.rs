//! The `server` module functions to spawn an [`axum_server`] which communicates over TLS.

mod auth;
mod response;
mod state;

use core::{marker::PhantomData, time::Duration};
use std::net::SocketAddr;

use axum::{
	error_handling::HandleErrorLayer,
	http::StatusCode,
	routing::{self, MethodRouter},
	BoxError,
	Router,
};
use axum_server::tls_rustls::RustlsConfig;
pub use response::{LoginResponse, LogoutResponse, Response};
use sqlx::{Connection, Database, Executor, Transaction};
pub use state::State;
use tower::{timeout, ServiceBuilder};
use tower_http::{compression::CompressionLayer, trace::TraceLayer};
use winvoice_adapter::{Deletable, Retrievable, Updatable};

use self::auth::InitializableWithAuthorization;
use crate::{api::schema::Adapter, DynResult};

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
		state: State<Db>,
		session_ttl: Duration,
		timeout: Option<Duration>,
	) -> DynResult<()>
	where
		A: Adapter<Db = Db> + InitializableWithAuthorization,
	{
		let router = Self::router::<A>(state, session_ttl, timeout).await?;
		axum_server::bind_rustls(self.address, self.tls).serve(router.into_make_service()).await?;
		Ok(())
	}

	/// Create a new [`MethodRouter`] with [`delete`](routing::delete) and [`patch`](routing::patch)
	/// preconfigured, since those are common among all Winvoice entities.
	fn route<TEntity>() -> MethodRouter<State<Db>>
	where
		TEntity: Deletable<Db = Db> + Retrievable<Db = Db> + Updatable<Db = Db>,
	{
		routing::delete(|| async { todo("Delete method not implemented") })
			.patch(|| async { todo("Update method not implemented") })
	}

	/// Create the [`Router`] that will be used by the [`Server`].
	async fn router<A>(
		state: State<Db>,
		session_ttl: Duration,
		timeout: Option<Duration>,
	) -> DynResult<Router>
	where
		A: Adapter<Db = Db> + InitializableWithAuthorization,
	{
		A::init_with_auth(state.pool()).await?;

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
								format!("Unhandled internal error: {}", err),
							),
						}
					}))
					.timeout(t),
			);
		}

		Ok(router
			.layer(CompressionLayer::new())
			.layer(TraceLayer::new_for_http())
			.route("/login", routing::put(|| async { todo("login") }))
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
