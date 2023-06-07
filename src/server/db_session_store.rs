//! Contains the structure which is used to store session data.

mod clone;
mod initializable;
mod session_store;

use sqlx::{Database, Executor, Pool, Result};
use winvoice_adapter::Initializable;

/// A session storer which is agnostic over the given `Db`.
#[derive(Debug)]
pub struct DbSessionStore<Db>
where
	Db: Database,
{
	/// The [`Pool`] of connections to the [`Database`].
	pool: Pool<Db>,
}

impl<Db> DbSessionStore<Db>
where
	Db: Database,
{
	/// Get the current [`Connection`](sqlx::Connection).
	pub fn connection(&self) -> impl Executor<'_, Database = Db>
	where
		for<'conn> &'conn Pool<Db>: Executor<'conn, Database = Db>,
	{
		&self.pool
	}

	/// Create a new [`DbSessionStore`].
	pub const fn new(pool: Pool<Db>) -> Self
	{
		Self { pool }
	}
}

impl<Db> DbSessionStore<Db>
where
	Db: Database,
	DbSessionStore<Db>: Initializable<Db = Db>,
{
	pub async fn init(&self) -> Result<()>
	{
		<DbSessionStore<Db> as Initializable>::init(&self.pool).await?;
		Ok(())
	}
}
