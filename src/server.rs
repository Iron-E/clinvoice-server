use std::net::SocketAddr;

use axum_server::tls_rustls::RustlsConfig;
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
use sqlx::{Connection, Database, Executor, Transaction};

use crate::{login::Login, router::Router, DynResult};

pub struct Server
{
	/// The IP address to bind the Winvoice server to.
	pub address: SocketAddr,

	/// The configuration for the TLS protocol via [rustls](axum_server::tls_rustls).
	pub tls: RustlsConfig,
}

impl Server
{
	pub async fn serve<C, E, J, L, O, T, X, Db>(
		self,
		connect_options: <Db::Connection as Connection>::Options,
	) -> DynResult<()>
	where
		Db: Database,
		<Db::Connection as Connection>::Options: Login + Clone,
		for<'connection> &'connection mut Db::Connection: Executor<'connection, Database = Db>,
		for<'connection> &'connection mut Transaction<'connection, Db>:
			Executor<'connection, Database = Db>,
		C: Deletable<Db = Db> + ContactAdapter,
		E: Deletable<Db = Db> + EmployeeAdapter,
		J: Deletable<Db = Db> + JobAdapter,
		L: Deletable<Db = Db> + LocationAdapter,
		O: Deletable<Db = Db> + OrganizationAdapter,
		T: Deletable<Db = Db> + TimesheetAdapter,
		X: Deletable<Db = Db> + ExpensesAdapter,
	{
		axum_server::bind_rustls(self.address, self.tls)
			.serve(Router::new(connect_options).route::<C, E, J, L, O, T, X>().into_make_service())
			.await?;

		Ok(())
	}
}
