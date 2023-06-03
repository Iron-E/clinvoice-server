//! Contains the structure which is used to store session data.

mod clone;
mod session_store;

use sqlx::{Database, Executor, Pool, Result};

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
	for<'c> &'c Pool<Db>: Executor<'c, Database = Db>,
{
	/// Get the current [`Connection`](sqlx::Connection).
	pub fn connection(&self) -> impl Executor<'_, Database = Db>
	{
		&self.pool
	}
}

#[cfg(feature = "postgres")]
impl DbSessionStore<sqlx::Postgres>
{
	pub async fn new(pool: sqlx::PgPool) -> Result<Self>
	{
		sqlx::query!(
			"CREATE TABLE IF NOT EXISTS sessions
			(
				id text NOT NULL PRIMARY KEY,
				expiry timestamptz,
				session json NOT NULL
			);"
		)
		.execute(&pool)
		.await?;

		Ok(Self { pool })
	}
}
