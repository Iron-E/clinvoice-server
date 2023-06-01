//! Contains an implementation of [`UserAdapter`] for [`PgUser`]

use sqlx::{Error, Executor, Postgres, Result};
use winvoice_adapter_postgres::fmt::DateTimeExt;
use winvoice_schema::{
	chrono::{DateTime, Duration, Utc},
	Employee,
};

use super::PgUser;
use crate::api::schema::{Role, User, UserAdapter};

#[async_trait::async_trait]
impl UserAdapter for PgUser
{
	async fn create<'connection, Conn>(
		connection: Conn,
		employee: Option<Employee>,
		password: String,
		role: Role,
		username: String,
	) -> Result<User>
	where
		Conn: Executor<'connection, Database = Postgres>,
	{
		let mut user = User::new(employee, 0, password, role, username)
			.map_err(|e| Error::Decode(e.into()))?;

		let row = sqlx::query!(
			"INSERT INTO users (employee_id, password, password_expires, role_id, username) \
			 VALUES ($1, $2, $3, $4, $5) RETURNING id;",
			user.employee_id(),
			user.password(),
			user.password_expires(),
			user.role_id(),
			user.username(),
		)
		.fetch_one(connection)
		.await?;

		user.set_id(row.id);
		Ok(user.pg_sanitize())
	}
}

#[cfg(test)]
mod tests
{
	use std::collections::HashMap;

	use pretty_assertions::{assert_eq, assert_str_eq};
	use sqlx::{PgPool, Transaction};
	use winvoice_adapter::{schema::EmployeeAdapter, Retrievable};
	use winvoice_adapter_postgres::schema::PgEmployee;
	use winvoice_schema::Id;

	use super::{DateTime, Employee, PgUser, Postgres, Result, Role, User, UserAdapter};
	use crate::{
		api::schema::postgres::role::role_adapter::tests as role,
		dyn_result::DynResult,
		utils::connect_pg,
	};

	/// `SELECT` from `users` where the joel or peggy id matches.
	macro_rules! select {
		($tx:expr, $joel_id:expr, $peggy_id:expr) => {
			sqlx::query!("SELECT * FROM users WHERE id IN ($1, $2)", $joel_id, $peggy_id)
				.fetch_all($tx)
				.await
				.map(|v: Vec<_>| v.into_iter().map(|r| (r.id, r)).collect())?
		};
	}

	/// # Returns
	///
	/// `(joel, peggy)`.
	async fn setup(tx: &mut Transaction<'_, Postgres>) -> Result<(User, User)>
	{
		let (admin, guest) = role::setup(&mut *tx).await?;
		let joel = PgUser::create(
			&mut *tx,
			None,
			"foobar".into(),
			guest,
			format!("joel{}", rand::random::<[char; 8]>().into_iter().collect::<String>()),
		)
		.await?;

		let margaret =
			PgEmployee::create(&mut *tx, "margaret".into(), "Hired".into(), "Manager".into())
				.await?;

		let peggy = PgUser::create(
			&mut *tx,
			margaret.into(),
			"asldkj".into(),
			admin,
			format!("peggy{}", rand::random::<[char; 8]>().into_iter().collect::<String>()),
		)
		.await?;

		Ok((joel, peggy))
	}

	/// Cleans up the [`setup`]
	pub async fn tear_down(pool: &PgPool, joel: User, peggy: User) -> Result<()>
	{
		sqlx::query!("DELETE FROM users WHERE id IN ($1, $2);", joel.id(), peggy.id())
			.execute(pool)
			.await?;

		role::tear_down(pool, joel.role_id(), peggy.role_id()).await
	}

	#[tokio::test]
	async fn create() -> DynResult<()>
	{
		let pool = connect_pg();
		let mut tx = pool.begin().await?;
		let (joel, peggy) = setup(&mut tx).await?;
		let rows: HashMap<_, _> = select!(&mut tx, joel.id(), peggy.id());

		let joel_row = rows
			.get(&joel.id())
			.ok_or_else(|| "The `joel` row does not exist in the database".to_owned())?;
		let peggy_row = rows
			.get(&peggy.id())
			.ok_or_else(|| "The `peggy` row does not exist in the database".to_owned())?;

		assert_eq!(joel.employee_id(), joel_row.employee_id);
		assert_eq!(joel.id(), joel_row.id);
		assert_eq!(joel.password(), joel_row.password);
		assert_eq!(joel.password_expires(), joel_row.password_expires);
		assert_eq!(joel.role_id(), joel_row.role_id);
		assert_eq!(joel.username(), joel_row.username);
		assert_eq!(peggy.employee_id(), peggy_row.employee_id);
		assert_eq!(peggy.id(), peggy_row.id);
		assert_eq!(peggy.password(), peggy_row.password);
		assert_eq!(peggy.password_expires(), peggy_row.password_expires);
		assert_eq!(peggy.role_id(), peggy_row.role_id);
		assert_eq!(peggy.username(), peggy_row.username);
		assert_str_eq!(joel.password(), joel_row.password);
		assert_str_eq!(joel.username(), joel_row.username);
		assert_str_eq!(peggy.password(), peggy_row.password);
		assert_str_eq!(peggy.username(), peggy_row.username);

		Ok(())
	}

	#[tokio::test]
	async fn retrieve() -> DynResult<()>
	{
		let pool = connect_pg();
		let mut tx = pool.begin().await?;
		let (joel, peggy) = setup(&mut tx).await?;
		tx.commit().await?;

		#[rustfmt::skip]
		let peggy_row = PgUser::retrieve(&pool, peggy.id().into()).await.map(|mut v| v.remove(0))?;
		let joel_row = PgUser::retrieve(&pool, joel.id().into()).await.map(|mut v| v.remove(0))?;

		assert_eq!(joel.employee_id(), joel_row.employee_id());
		assert_eq!(joel.id(), joel_row.id());
		assert_eq!(joel.password(), joel_row.password());
		assert_eq!(joel.password_expires(), joel_row.password_expires());
		assert_eq!(joel.role_id(), joel_row.role_id());
		assert_eq!(joel.username(), joel_row.username());
		assert_eq!(peggy.employee_id(), peggy_row.employee_id());
		assert_eq!(peggy.id(), peggy_row.id());
		assert_eq!(peggy.password(), peggy_row.password());
		assert_eq!(peggy.password_expires(), peggy_row.password_expires());
		assert_eq!(peggy.role_id(), peggy_row.role_id());
		assert_eq!(peggy.username(), peggy_row.username());
		assert_str_eq!(joel.password(), joel_row.password());
		assert_str_eq!(joel.username(), joel_row.username());
		assert_str_eq!(peggy.password(), peggy_row.password());
		assert_str_eq!(peggy.username(), peggy_row.username());

		Ok(())
	}
}
