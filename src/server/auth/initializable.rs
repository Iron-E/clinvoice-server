//! Contains the [`InitUsersTable`] trait, and implementations for various [`Database`]s.

use sqlx::{Database, Pool, Result};

/// Implementors of this trait are marked as able to both Initialize the base Winvoice
/// tables (see [`winvoice_adapter`]), but also an extended set of tables used by the
/// [`winvoice_server`] to [authorize](super) its users.
#[async_trait::async_trait]
pub trait Initializable: winvoice_adapter::Initializable
{
	/// The [`Database`] that will be used to initialize the authorization tables.
	type Db: Database;

	/// Initialize the [`auth`](super) tables on the [`Database`]
	async fn init(pool: &Pool<Self::Db>) -> Result<()>;
}

#[cfg(feature = "postgres")]
mod postgres
{
	use sqlx::Postgres;
	use winvoice_adapter_postgres::PgSchema;

	#[async_trait::async_trait]
	impl Initializable for PgSchema
	{
		type Db: Postgres;

		async fn init(pool: &Pool<Self::Db>) -> Result<()>
		{
			let mut tx = pool.begin().await?;
			Self::init(&mut tx).await?;

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
			.execute(&mut tx)
			.await?;

			sqlx::query!(
				"CREATE TABLE IF NOT EXISTS roles
				(
					id bigint PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
					name text NOT NULL UNIQUE,
					password_ttl interval,
				);"
			)
			.execute(&mut tx)
			.await?;

			tx.commit().await
		}
	}
}
