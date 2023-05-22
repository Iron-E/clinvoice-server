//! The `server` module functions to spawn an [`axum_server`] which communicates over TLS.

mod response;
mod sessions;

use core::time::Duration;
use std::net::SocketAddr;

use axum::{
	error_handling::HandleErrorLayer,
	http::StatusCode,
	routing::{self, MethodRouter},
	BoxError,
	Router,
};
use axum_server::tls_rustls::RustlsConfig;
pub use response::Response;
use sessions::{Login, SessionManager};
use sqlx::{Connection, Database, Executor, Transaction};
use tower::{timeout, ServiceBuilder};
use tower_http::compression::CompressionLayer;
use winvoice_adapter::{
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
	Updatable,
};

use crate::DynResult;

/// A Winvoice server.
#[derive(Clone, Debug)]
pub struct Server<Db>
where
	Db: Database,
	Db::Connection: core::fmt::Debug,
	<Db::Connection as Connection>::Options: Clone,
{
	/// The [`SocketAddr`] that self server is bound to.
	address: SocketAddr,

	/// The [`SessionManager`] which keeps track of active connections and logins while the server
	/// is open.
	session_manager: SessionManager<Db>,

	/// The amount of time to run operations on the server before cancelling them.
	timeout: Option<Duration>,

	/// The TLS configuration.
	tls: RustlsConfig,
}

impl<Db> Server<Db>
where
	Db: Database,
	Db::Connection: core::fmt::Debug,
	<Db::Connection as Connection>::Options: Clone + Login,
	for<'connection> &'connection mut Db::Connection: Executor<'connection, Database = Db>,
	for<'connection> &'connection mut Transaction<'connection, Db>:
		Executor<'connection, Database = Db>,
{
	/// Create a new [`Server`]
	pub fn new(
		address: SocketAddr,
		connect_options: <Db::Connection as Connection>::Options,
		session_expire: Duration,
		session_idle: Duration,
		timeout: Option<Duration>,
		tls: RustlsConfig,
	) -> Self
	{
		Self {
			address,
			session_manager: SessionManager::new(connect_options, session_idle, session_expire),
			timeout,
			tls,
		}
	}

	/// Create an [`Router`] based on the `connect_options`.
	///
	/// Operations `timeout`, if specified.
	pub async fn serve<C, E, J, L, O, T, X>(self) -> DynResult<()>
	where
		C: Deletable<Db = Db> + ContactAdapter,
		E: Deletable<Db = Db> + EmployeeAdapter,
		J: Deletable<Db = Db> + JobAdapter,
		L: Deletable<Db = Db> + LocationAdapter,
		O: Deletable<Db = Db> + OrganizationAdapter,
		T: Deletable<Db = Db> + TimesheetAdapter,
		X: Deletable<Db = Db> + ExpensesAdapter,
	{
		axum_server::bind_rustls(self.address, self.tls)
			.serve(
				Self::router::<C, E, J, L, O, T, X>(self.session_manager, self.timeout)
					.into_make_service(),
			)
			.await?;

		Ok(())
	}

	/// Create a new [`MethodRouter`] with [`delete`](routing::delete) and [`patch`](routing::patch)
	/// preconfigured, since those are common among all Winvoice entities.
	fn route<T>() -> MethodRouter<SessionManager<Db>>
	where
		T: Deletable<Db = Db> + Updatable<Db = Db>,
	{
		routing::delete(|| async { todo("Delete method not implemented") })
			.patch(|| async { todo("Update method not implemented") })
	}

	/// Create the [`Router`] that will be used by the [`Server`].
	fn router<C, E, J, L, O, T, X>(
		session_manager: SessionManager<Db>,
		timeout: Option<Duration>,
	) -> Router
	where
		C: Deletable<Db = Db> + ContactAdapter,
		E: Deletable<Db = Db> + EmployeeAdapter,
		J: Deletable<Db = Db> + JobAdapter,
		L: Deletable<Db = Db> + LocationAdapter,
		O: Deletable<Db = Db> + OrganizationAdapter,
		T: Deletable<Db = Db> + TimesheetAdapter,
		X: Deletable<Db = Db> + ExpensesAdapter,
	{
		let mut router = Router::new().layer(CompressionLayer::new());

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

		router
			.route("/login", routing::put(sessions::login))
			.route("/logout", routing::put(sessions::logout))
			.route(
				"/contact",
				Self::route::<C>()
					.get(|| async { todo("contact retrieve") })
					.post(|| async { todo("contact create") }),
			)
			.route(
				"/employee",
				Self::route::<E>()
					.get(|| async { todo("employee retrieve") })
					.post(|| async { todo("employee create") }),
			)
			.route(
				"/expense",
				Self::route::<X>()
					.get(|| async { todo("expense retrieve") })
					.post(|| async { todo("expense create") }),
			)
			.route(
				"/job",
				Self::route::<J>()
					.get(|| async { todo("job retrieve") })
					.post(|| async { todo("job create") }),
			)
			.route(
				"/location",
				Self::route::<L>()
					.get(|| async { todo("location retrieve") })
					.post(|| async { todo("location create") }),
			)
			.route(
				"/organization",
				Self::route::<O>()
					.get(|| async { todo("organization retrieve") })
					.post(|| async { todo("organization create") }),
			)
			.route(
				"/timesheet",
				Self::route::<T>()
					.get(|| async { todo("timesheet retrieve") })
					.post(|| async { todo("timesheet create") }),
			)
			.with_state(session_manager)
	}
}

const fn todo(msg: &'static str) -> (StatusCode, &'static str)
{
	(StatusCode::NOT_IMPLEMENTED, msg)
}
