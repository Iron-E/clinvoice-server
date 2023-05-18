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
	/// The [`ConnectOptions`](sqlx::ConnectOptions) used to connect to the database.
	pub connect_options: <Db::Connection as Connection>::Options,
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
		todo!("Start the server with axum")
	}
}
