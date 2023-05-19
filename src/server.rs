use core::time::Duration;
use std::net::SocketAddr;

use axum_server::tls_rustls::RustlsConfig;
use sqlx::{Connection, Database, Executor, Transaction};
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

use crate::{login::Login, router::Router, DynResult};

pub struct Server
{
	/// The IP address to bind the Winvoice server to.
	pub address: SocketAddr,

	/// The configuration for the TLS protocol via [rustls](axum_server::tls_rustls).
	pub tls: RustlsConfig,

	/// If [`Some`], the amount of time to run commands on the server before timing out.
	pub timeout: Option<Duration>,
}

impl Server
{
	/// Start a [`Server`](axum::Server) using the `connect_options`.
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
			.serve(
				Router::axum::<C, E, J, L, O, T, X>(connect_options, self.timeout)
					.into_make_service(),
			)
			.await?;

		Ok(())
	}
}
