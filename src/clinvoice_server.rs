use std::net::SocketAddr;

use axum::{routing, Router, Server};
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

pub struct CLInvoiceServer<Db>
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

impl<Db> CLInvoiceServer<Db>
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
		let router = Router::new().route("/", routing::get(|| async { "Hello World!" }));

		axum_server::bind_rustls(self.address, self.tls).serve(router.into_make_service()).await?;
		Ok(())
	}
}
