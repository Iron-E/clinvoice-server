//! Contains an implementation of [`UserAdapter`] for [`PgUser`]

use async_session::chrono::{DateTime, Utc};
use sqlx::{Executor, Postgres, Result};
use winvoice_schema::Employee;

use super::PgUser;
use crate::api::schema::{Role, User, UserAdapter};

#[async_trait::async_trait]
impl UserAdapter for PgUser
{
	async fn create<'connection, Conn>(
		connection: Conn,
		employee: Option<Employee>,
		password: String,
		password_expires: Option<DateTime<Utc>>,
		role: Role,
		username: String,
	) -> Result<User>
	where
		Conn: Executor<'connection, Database = Postgres>,
	{
		let employee_id = employee.map(|e| e.id);
		let role_id = role.id();

		let row = sqlx::query!(
			"INSERT INTO users (employee_id, password, password_expires, role_id, username) \
			 VALUES ($1, $2, $3, $4, $5) RETURNING id;",
			employee_id,
			password,
			password_expires,
			role_id,
			username,
		)
		.fetch_one(connection)
		.await?;

		Ok(User::new(employee_id, row.id, password, password_expires, role_id, username))
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
