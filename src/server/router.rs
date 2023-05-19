//! Manages routing requests to API endpoints to the various internal handlers.

use core::time::Duration;

use axum::{
	error_handling::HandleErrorLayer,
	http::StatusCode,
	routing::{self, MethodRouter},
	BoxError,
	Router as AxumRouter,
};
use sqlx::{pool::PoolOptions, Connection, Database, Executor, Pool, Result, Transaction};
use tower::{
	timeout::{self, TimeoutLayer},
	ServiceBuilder,
};
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

use super::Login;

static IDLE_TIMEOUT: Duration = Duration::from_secs(300);

/// A router for the Winvoice server, which handles operations on all the API endpoints.
pub struct Router<Db>
where
	Db: Database,
{
	connect_options: <Db::Connection as Connection>::Options,
}

// TODO: add std::cell::OnceLock of HashMap<UserName, PgPool> which caches active user connections

impl<Db> Router<Db>
where
	Db: Database,
	<Db::Connection as Connection>::Options: Login + Clone,
	for<'connection> &'connection mut Db::Connection: Executor<'connection, Database = Db>,
	for<'connection> &'connection mut Transaction<'connection, Db>:
		Executor<'connection, Database = Db>,
{
	/// Create an [`axum::Router`] based on the `connect_options`.
	///
	/// Operations `timeout`, if specified.
	pub fn axum<C, E, J, L, O, T, X>(
		connect_options: <Db::Connection as Connection>::Options,
		timeout: Option<Duration>,
	) -> AxumRouter
	where
		C: Deletable<Db = Db> + ContactAdapter,
		E: Deletable<Db = Db> + EmployeeAdapter,
		J: Deletable<Db = Db> + JobAdapter,
		L: Deletable<Db = Db> + LocationAdapter,
		O: Deletable<Db = Db> + OrganizationAdapter,
		T: Deletable<Db = Db> + TimesheetAdapter,
		X: Deletable<Db = Db> + ExpensesAdapter,
	{
		let mut router = AxumRouter::new()
			.layer(CompressionLayer::new())
			.layer(ValidateRequestHeaderLayer::accept("application/json"));

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
					.layer(TimeoutLayer::new(t)),
			);
		}

		let this = Self { connect_options };
		router
			.route("/login", routing::get(|| async { todo("login") }))
			.route(
				"/contact",
				this.route::<C>()
					.get(|| async { todo("contact retrieve") })
					.post(|| async { todo("contact create") }),
			)
			.route(
				"/employee",
				this.route::<E>()
					.get(|| async { todo("employee retrieve") })
					.post(|| async { todo("employee create") }),
			)
			.route(
				"/expense",
				this.route::<X>()
					.get(|| async { todo("expense retrieve") })
					.post(|| async { todo("expense create") }),
			)
			.route(
				"/job",
				this.route::<J>()
					.get(|| async { todo("job retrieve") })
					.post(|| async { todo("job create") }),
			)
			.route(
				"/location",
				this.route::<L>()
					.get(|| async { todo("location retrieve") })
					.post(|| async { todo("location create") }),
			)
			.route(
				"/organization",
				this.route::<O>()
					.get(|| async { todo("organization retrieve") })
					.post(|| async { todo("organization create") }),
			)
			.route(
				"/timesheet",
				this.route::<T>()
					.get(|| async { todo("timesheet retrieve") })
					.post(|| async { todo("timesheet create") }),
			)
	}

	/// Create a new [`Pool`] which attempts to establish a connection with the database that this
	/// [`Router`] has been instructed to communicate with.
	///
	/// Uses `username` and `password` as credentials for the new connection.
	async fn login(&self, username: &str, password: &str) -> Result<Pool<Db>>
	{
		PoolOptions::new()
			.idle_timeout(IDLE_TIMEOUT)
			.max_connections(1)
			.connect_with(self.connect_options.clone().login(username, password))
			.await
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
