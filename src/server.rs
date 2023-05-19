//! The `server` module functions to spawn an [`axum_server`] which communicates over TLS.

mod router;
mod session;

use core::time::Duration;
use std::net::SocketAddr;

use axum_server::tls_rustls::RustlsConfig;
use router::Router;
use session::Login;
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

use crate::DynResult;

/// Bind a [`Server`](axum::Server) to the `address` which communicates over `tls`.
///
/// * Connection to the Winvoice database is managed by the `connect_options`.
/// * Operations `timeout`, if [`Some`] value is specified.
pub async fn serve<C, E, J, L, O, T, X, Db>(
	address: SocketAddr,
	connect_options: <Db::Connection as Connection>::Options,
	session_expire: Option<Duration>,
	timeout: Option<Duration>,
	tls: RustlsConfig,
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
	axum_server::bind_rustls(address, tls)
		.serve(
			Router::axum::<C, E, J, L, O, T, X>(connect_options, session_expire, timeout)
				.into_make_service(),
		)
		.await?;

	Ok(())
}
