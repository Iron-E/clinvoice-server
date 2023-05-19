//! The `server` module functions to spawn an [`axum_server`] which communicates over TLS.

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
use sessions::{Login, SessionManager};
use sqlx::{Connection, Database, Executor, Transaction};
use tower::{timeout, ServiceBuilder};
use tower_http::{compression::CompressionLayer, validate_request::ValidateRequestHeaderLayer};
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
pub struct Server<Db>
where
	Db: Database,
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
	<Db::Connection as Connection>::Options: Login + Clone,
	for<'connection> &'connection mut Db::Connection: Executor<'connection, Database = Db>,
	for<'connection> &'connection mut Transaction<'connection, Db>:
		Executor<'connection, Database = Db>,
{
	/// Create a new [`Server`]
	pub fn new(
		address: SocketAddr,
		connect_options: <Db::Connection as Connection>::Options,
		session_expire: Option<Duration>,
		timeout: Option<Duration>,
		tls: RustlsConfig,
	) -> Self
	{
		Self {
			address,
			session_manager: SessionManager::new(
				connect_options,
				Duration::from_secs(300).into(),
				session_expire,
			),
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
		let mut router = Router::new()
			.layer(CompressionLayer::new())
			.layer(ValidateRequestHeaderLayer::accept("application/json"));

		if let Some(t) = self.timeout
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

		router = self
			.session_manager
			.route_logout(self.session_manager.route_login(router))
			.route(
				"/contact",
				self.route::<C>()
					.get(|| async { todo("contact retrieve") })
					.post(|| async { todo("contact create") }),
			)
			.route(
				"/employee",
				self.route::<E>()
					.get(|| async { todo("employee retrieve") })
					.post(|| async { todo("employee create") }),
			)
			.route(
				"/expense",
				self.route::<X>()
					.get(|| async { todo("expense retrieve") })
					.post(|| async { todo("expense create") }),
			)
			.route(
				"/job",
				self.route::<J>()
					.get(|| async { todo("job retrieve") })
					.post(|| async { todo("job create") }),
			)
			.route(
				"/location",
				self.route::<L>()
					.get(|| async { todo("location retrieve") })
					.post(|| async { todo("location create") }),
			)
			.route(
				"/organization",
				self.route::<O>()
					.get(|| async { todo("organization retrieve") })
					.post(|| async { todo("organization create") }),
			)
			.route(
				"/timesheet",
				self.route::<T>()
					.get(|| async { todo("timesheet retrieve") })
					.post(|| async { todo("timesheet create") }),
			);

		axum_server::bind_rustls(self.address, self.tls).serve(router.into_make_service()).await?;
		Ok(())
	}

	/// Create a new [`MethodRouter`] with [`delete`](routing::delete) and [`patch`](routing::patch)
	/// preconfigured, since those are common among all Winvoice entities.
	fn route<T>(&self) -> MethodRouter
	where
		T: Deletable<Db = Db> + Updatable<Db = Db>,
	{
		routing::delete(|| async { todo("Delete method not implemented") })
			.patch(|| async { todo("Update method not implemented") })
	}
}

const fn todo(msg: &'static str) -> (StatusCode, &'static str)
{
	(StatusCode::NOT_IMPLEMENTED, msg)
}
