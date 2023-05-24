//! Contains the [`InitUsersTable`] trait, and implementations for various [`Database`]s.

use sqlx::{Database, Executor, Result};

/// Initialize the `users`
#[async_trait::async_trait]
pub trait InitUsersTable: Database
{
	/// Initialize the `users` table on the [`Database`]
	async fn init_users_table<'conn, C>(connection: C) -> Result<()>
	where
		C: Executor<'conn, Database = Self>;
}

#[cfg(feature = "postgres")]
#[async_trait::async_trait]
impl InitUsersTable for sqlx::Postgres
{
	async fn init_users_table<'conn, C>(connection: C) -> Result<()>
	where
		C: Executor<'conn, Database = Self>,
	{
		Ok(())
	}
}
