use core::time::Duration;

use axum::{
	http::StatusCode,
	routing::{self, MethodRouter},
	Router as AxumRouter,
};
use sqlx::{pool::PoolOptions, Connection, Database, Executor, Pool, Transaction};
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

use crate::login::Login;

static IDLE_TIMEOUT: Duration = Duration::from_secs(300);

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
	pub fn axum<C, E, J, L, O, T, X>(
		connect_options: <Db::Connection as Connection>::Options,
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
		let this = Self { connect_options };

		let contact_route = this.route::<C>();
		let employee_route = this.route::<E>();
		let expense_route = this.route::<X>();
		let location_route = this.route::<L>();
		let job_route = this.route::<J>();
		let organization_route = this.route::<O>();
		let timesheet_route = this.route::<T>();

		AxumRouter::new()
			.route("/", routing::get(|| async { (StatusCode::NOT_FOUND, "CUSTOM ERROR") }))
			.route(
				"/contact",
				contact_route
					.get(|| async { todo!("contact retrieve") })
					.post(|| async { todo!("contact create") }),
			)
			.route(
				"/employee",
				employee_route
					.get(|| async { todo!("employee retrieve") })
					.post(|| async { todo!("employee create") }),
			)
			.route(
				"/expense",
				expense_route
					.get(|| async { todo!("expense retrieve") })
					.post(|| async { todo!("expense create") }),
			)
			.route(
				"/job",
				job_route
					.get(|| async { todo!("job retrieve") })
					.post(|| async { todo!("job create") }),
			)
			.route(
				"/location",
				location_route
					.get(|| async { todo!("location retrieve") })
					.post(|| async { todo!("location create") }),
			)
			.route(
				"/organization",
				organization_route
					.get(|| async { todo!("organization retrieve") })
					.post(|| async { todo!("organization create") }),
			)
			.route(
				"/timesheet",
				timesheet_route
					.get(|| async { todo!("timesheet retrieve") })
					.post(|| async { todo!("timesheet create") }),
			)
	}

	fn login(&self, username: &str, password: &str) -> Pool<Db>
	{
		PoolOptions::new()
			.idle_timeout(IDLE_TIMEOUT)
			.max_connections(1)
			.connect_lazy_with(self.connect_options.clone().login(username, password))
	}

	fn route<T>(&self) -> MethodRouter
	where
		T: Deletable<Db = Db> + Updatable<Db = Db>,
	{
		routing::delete(|| async { todo!("Implement delete method") })
			.patch(|| async { todo!("Implement delete method") })
	}
}
