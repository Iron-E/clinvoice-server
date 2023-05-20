//! The `server` module functions to spawn an [`axum_server`] which communicates over TLS.

mod response;
mod sessions;

use core::time::Duration;
use std::net::SocketAddr;

use axum::{
	error_handling::HandleErrorLayer,
	http::StatusCode,
	middleware,
	routing::{self, MethodRouter},
	BoxError,
	Json,
	Router,
};
use axum_server::tls_rustls::RustlsConfig;
pub use response::Response;
use sessions::SessionManager;
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

use crate::{
	api::{response::Login, Status, StatusCode as WinvoiceCode},
	DynResult,
};

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
	<Db::Connection as Connection>::Options: Clone + sessions::Login,
	for<'connection> &'connection mut Db::Connection: Executor<'connection, Database = Db>,
	for<'connection> &'connection mut Transaction<'connection, Db>:
		Executor<'connection, Database = Db>,
{
	/// Create a new [`Server`]
	pub fn new(
		address: SocketAddr,
		connect_options: <Db::Connection as Connection>::Options,
		session_expire: Option<Duration>,
		session_idle: Option<Duration>,
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
		let mut stateless_router = Router::new().layer(CompressionLayer::new());

		if let Some(t) = self.timeout
		{
			stateless_router = stateless_router.layer(
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

		let router = stateless_router
			.route("/login", routing::put(|| async { Response::new(
				StatusCode::OK,
				Login::new(WinvoiceCode::LoggedIn, None),
			)}))
			.route_layer(middleware::from_fn_with_state(self.session_manager.clone(), sessions::login))
			.route("/login", routing::put(|| async {
				Response::new(StatusCode::OK, Status::new(WinvoiceCode::LoggedIn, None))
			}))
			// .route_layer(middleware::from_fn_with_state(
			// 	self.session_manager.clone(),
			// 	sessions::logout_layer,
			// ))
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
			)
			.with_state(self.session_manager);

		axum_server::bind_rustls(self.address, self.tls).serve(router.into_make_service()).await?;
		Ok(())
	}

	/// Create a new [`MethodRouter`] with [`delete`](routing::delete) and [`patch`](routing::patch)
	/// preconfigured, since those are common among all Winvoice entities.
	fn route<T>(&self) -> MethodRouter<SessionManager<Db>>
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
