//! Contains the [`InitUsersTable`] trait, and implementations for various
//! [`Database`](sqlx::Database)s.

use sqlx::{Pool, Result};
use winvoice_adapter::Initializable;

/// Implementors of this trait are marked as able to both Initialize the base Winvoice
/// tables (see [`winvoice_adapter`]), but also an extended set of tables used by the
/// [`winvoice_server`] to [authorize](super) its users.
#[async_trait::async_trait]
pub trait InitializableWithAuthorization: Initializable
{
	/// Initialize the [`auth`](super) tables on the [`Database`](sqlx::Database)
	async fn init_with_auth(pool: &Pool<Self::Db>) -> Result<()>;
}

#[cfg(feature = "postgres")]
#[async_trait::async_trait]
impl InitializableWithAuthorization for winvoice_adapter_postgres::PgSchema
{
	async fn init_with_auth(pool: &sqlx::PgPool) -> Result<()>
	{
		let mut tx = pool.begin().await?;
		Self::init(&mut tx).await?;

		sqlx::query!(
			"CREATE TABLE IF NOT EXISTS roles
			(
				id bigint PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
				name text NOT NULL UNIQUE,
				password_ttl interval
			);"
		)
		.execute(&mut tx)
		.await?;

		sqlx::query!(
			"CREATE TABLE IF NOT EXISTS users
			(
				employee_id bigint REFERENCES employees(id),
				id bigint PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
				password text NOT NULL,
				password_expires timestamptz,
				role_id bigint NOT NULL REFERENCES roles(id),
				username text NOT NULL UNIQUE
			);"
		)
		.execute(&mut tx)
		.await?;

		tx.commit().await
	}
}
