use std::{fmt::Display, time::Duration};

use axum::{routing, Router as AxumRouter};
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
};
use sqlx::{pool::PoolOptions, Connection, Database, Executor, Pool, Transaction};

use crate::{dyn_result::DynResult, login::Login};

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
	async fn delete<T>(&self) -> DynResult<()>
	where
		T: Deletable<Db = Db>,
		<T as Deletable>::Entity: Clone + Display + Sync,
	{
		todo!("Implement delete method")
	}

	fn login(&self, username: &str, password: &str) -> Pool<Db>
	{
		PoolOptions::new()
			.idle_timeout(IDLE_TIMEOUT)
			.max_connections(1)
			.connect_lazy_with(self.connect_options.clone().login(username, password))
	}

	pub fn new(connect_options: <Db::Connection as Connection>::Options) -> Self
	{
		Self { connect_options }
	}

	pub fn route<C, E, J, L, O, T, X>(self) -> AxumRouter
	where
		C: Deletable<Db = Db> + ContactAdapter,
		E: Deletable<Db = Db> + EmployeeAdapter,
		J: Deletable<Db = Db> + JobAdapter,
		L: Deletable<Db = Db> + LocationAdapter,
		O: Deletable<Db = Db> + OrganizationAdapter,
		T: Deletable<Db = Db> + TimesheetAdapter,
		X: Deletable<Db = Db> + ExpensesAdapter,
	{
		AxumRouter::new()
			.route(
				"/contact",
				routing::delete(|| async { todo!("contact delete") })
					.get(|| async { todo!("contact retrieve") })
					.patch(|| async { todo!("contact update") })
					.post(|| async { todo!("contact create") }),
			)
			.route(
				"/employee",
				routing::delete(|| async { todo!("employee delete") })
					.get(|| async { todo!("employee retrieve") })
					.patch(|| async { todo!("employee update") })
					.post(|| async { todo!("employee create") }),
			)
			.route(
				"/expense",
				routing::delete(|| async { todo!("expense delete") })
					.get(|| async { todo!("expense retrieve") })
					.patch(|| async { todo!("expense update") })
					.post(|| async { todo!("expense create") }),
			)
			.route(
				"/job",
				routing::delete(|| async { todo!("job delete") })
					.get(|| async { todo!("job retrieve") })
					.patch(|| async { todo!("job update") })
					.post(|| async { todo!("job create") }),
			)
			.route(
				"/location",
				routing::delete(|| async { todo!("locationg delete") })
					.get(|| async { todo!("locationg retrieve") })
					.patch(|| async { todo!("locationg update") })
					.post(|| async { todo!("locationg create") }),
			)
			.route(
				"/organization",
				routing::delete(|| async { todo!("organization delete") })
					.get(|| async { todo!("organization retrieve") })
					.patch(|| async { todo!("organization update") })
					.post(|| async { todo!("organization create") }),
			)
			.route(
				"/timesheet",
				routing::delete(|| async { todo!("timesheet delete") })
					.get(|| async { todo!("timesheet retrieve") })
					.patch(|| async { todo!("timesheet update") })
					.post(|| async { todo!("timesheet create") }),
			)
	}
}
