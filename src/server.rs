use std::net::SocketAddr;

use axum::{routing, Router};
use axum_server::tls_rustls::RustlsConfig;
use clinvoice_adapter::{
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
use sqlx::{Connection, Database, Executor, Transaction};

use crate::DynResult;

pub struct Server<Db>
where
	Db: Database,
{
	/// The IP address to bind the CLInvoice server to.
	pub address: SocketAddr,

	/// The [`ConnectOptions`](sqlx::ConnectOptions) used to connect to the database.
	pub connect_options: <Db::Connection as Connection>::Options,

	/// The configuration for the TLS protocol via [rustls](axum_server::tls_rustls).
	pub tls: RustlsConfig,
}

impl<Db> Server<Db>
where
	Db: Database,
	<Db::Connection as Connection>::Options: Clone,
{
	pub async fn serve<CAdapter, EAdapter, JAdapter, LAdapter, OAdapter, TAdapter, XAdapter>(
		self,
	) -> DynResult<()>
	where
		CAdapter: Deletable<Db = Db> + ContactAdapter,
		EAdapter: Deletable<Db = Db> + EmployeeAdapter,
		JAdapter: Deletable<Db = Db> + JobAdapter,
		LAdapter: Deletable<Db = Db> + LocationAdapter,
		OAdapter: Deletable<Db = Db> + OrganizationAdapter,
		TAdapter: Deletable<Db = Db> + TimesheetAdapter,
		XAdapter: Deletable<Db = Db> + ExpensesAdapter,
		for<'connection> &'connection mut Db::Connection: Executor<'connection, Database = Db>,
		for<'connection> &'connection mut Transaction<'connection, Db>:
			Executor<'connection, Database = Db>,
	{
		async fn todo()
		{
			todo!()
		}

		let router = Router::new()
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
			);

		axum_server::bind_rustls(self.address, self.tls).serve(router.into_make_service()).await?;
		Ok(())
	}
}
