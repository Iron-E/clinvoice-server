//! Contains the [`InitUsersTable`] trait, and implementations for various [`Database`]s.

use sqlx::{Database, Pool, Result};

/// Initialize the `users`
#[async_trait::async_trait]
pub trait InitUsersTable: Database
{
	/// Initialize the `users` table on the [`Database`]
	async fn init_users_table(pool: &Pool<Db>) -> Result<()>;
}

#[cfg(feature = "postgres")]
#[async_trait::async_trait]
impl InitUsersTable for sqlx::Postgres
{
	async fn init_users_table(pool: &Pool<Db>) -> Result<()>
	{
		sqlx::query!(
			"CREATE TABLE IF NOT EXISTS users
			(
				employee_id bigint REFERENCES employees(id),
				id bigint PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
				password text NOT NULL,
				password_expires timestamptz,
				role text DEFAULT 'guest',
				username text NOT NULL UNIQUE,
			);"
		)
		.execute(connection)
		.await?;

		sqlx::query!(
			"CREATE TABLE IF NOT EXISTS roles
			(
				id bigint PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
				name text NOT NULL UNIQUE,
				password_ttl interval,
			);"
		)
		.execute(connection)
		.await?;

		Ok(())
	}
}
