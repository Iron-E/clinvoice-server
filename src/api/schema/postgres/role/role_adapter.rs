//! Contains an implementation of [`RoleAdapter`] for [`PgRole`]

use core::time::Duration;

use sqlx::{Executor, Postgres, Result};

use super::PgRole;
use crate::api::schema::{Role, RoleAdapter};

#[async_trait::async_trait]
impl RoleAdapter for PgRole
{
	async fn create<'connection, Conn>(
		connection: Conn,
		name: String,
		password_ttl: Option<Duration>,
	) -> Result<Role>
	where
		Conn: Executor<'connection, Database = Postgres>,
	{
		let row = sqlx::query!(
			"INSERT INTO roles (name, password_ttl) VALUES ($1, $2) RETURNING id;",
			name,
			password_ttl as _,
		)
		.fetch_one(connection)
		.await?;

		Ok(Role::new(row.id, name, password_ttl))
	}
}

#[cfg(test)]
mod tests
{
	use pretty_assertions::assert_eq;

	#[tokio::test]
	async fn create()
	{
		todo!("Write test")
	}
}
