//! Contains the [`InitUsersTable`] trait, and implementations for various
//! [`Database`](sqlx::Database)s.

use sqlx::{Pool, Result};
use winvoice_adapter::Initializable;

#[cfg(feature = "postgres")]
use crate::schema::postgres::{PgRole, PgUser};
use crate::schema::{RoleAdapter, UserAdapter};

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
				id uuid PRIMARY KEY,
				name text NOT NULL UNIQUE,
				password_ttl interval
			);"
		)
		.execute(&mut tx)
		.await?;

		sqlx::query!(
			"CREATE TABLE IF NOT EXISTS users
			(
				id uuid PRIMARY KEY,
				employee_id uuid REFERENCES employees(id),
				password text NOT NULL,
				password_set timestamp NOT NULL,
				role_id uuid NOT NULL REFERENCES roles(id),
				username text NOT NULL UNIQUE
			);"
		)
		.execute(&mut tx)
		.await?;

		let has_rows = sqlx::query!("SELECT * FROM users LIMIT 1").fetch_optional(&mut tx).await?;
		if has_rows.is_none()
		{
			let role = PgRole::create(&mut tx, "admin".into(), None).await?;
			PgUser::create(&mut tx, None, "password".into(), role, "admin".into()).await?;
		}

		tx.commit().await
	}
}
