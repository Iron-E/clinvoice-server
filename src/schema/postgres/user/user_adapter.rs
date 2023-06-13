//! Contains an implementation of [`UserAdapter`] for [`PgUser`]

use sqlx::{Error, Executor, Postgres, Result};
use winvoice_adapter_postgres::fmt::DateTimeExt;
use winvoice_schema::Employee;

use super::PgUser;
use crate::schema::{Role, User, UserAdapter};

#[async_trait::async_trait]
impl UserAdapter for PgUser
{
	#[tracing::instrument(level = "trace", skip_all, err)]
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
			user.employee().map(|e| e.id),
			user.password(),
			user.password_expires().map(|d| d.naive_utc()),
			user.role().id(),
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
	use core::time::Duration;
	use std::collections::HashMap;

	use pretty_assertions::{assert_eq, assert_str_eq};
	use sqlx::Transaction;
	use tracing_test::traced_test;
	use winvoice_adapter::{schema::EmployeeAdapter, Deletable, Retrievable, Updatable};
	use winvoice_adapter_postgres::schema::PgEmployee;

	use super::{DateTimeExt, PgUser, Postgres, Result, User, UserAdapter};
	use crate::{
		dyn_result::DynResult,
		schema::{
			postgres::{role::role_adapter::tests as role, PgRole},
			RoleAdapter,
		},
		utils::{connect_pg, different_string, random_string},
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
			format!("joel{}", random_string()),
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
			format!("peggy{}", random_string()),
		)
		.await?;

		Ok((joel, peggy))
	}

	#[tokio::test]
	#[traced_test]
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

		assert_eq!(joel.employee().map(|e| e.id), joel_row.employee_id);
		assert_eq!(joel.id(), joel_row.id);
		assert_eq!(joel.password(), joel_row.password);
		assert_eq!(joel.password_expires().map(|d| d.naive_utc()), joel_row.password_expires);
		assert_eq!(joel.role().id(), joel_row.role_id);
		assert_eq!(joel.username(), joel_row.username);
		assert_eq!(peggy.employee().map(|e| e.id), peggy_row.employee_id);
		assert_eq!(peggy.id(), peggy_row.id);
		assert_eq!(peggy.password(), peggy_row.password);
		assert_eq!(peggy.password_expires().map(|d| d.naive_utc()), peggy_row.password_expires);
		assert_eq!(peggy.role().id(), peggy_row.role_id);
		assert_eq!(peggy.username(), peggy_row.username);
		assert_str_eq!(joel.password(), joel_row.password);
		assert_str_eq!(joel.username(), joel_row.username);
		assert_str_eq!(peggy.password(), peggy_row.password);
		assert_str_eq!(peggy.username(), peggy_row.username);

		Ok(())
	}

	#[tokio::test]
	#[traced_test]
	async fn delete() -> DynResult<()>
	{
		let pool = connect_pg();
		let mut tx = pool.begin().await?;
		let (joel, peggy) = setup(&mut tx).await?;

		PgUser::delete(&mut tx, [&joel].into_iter()).await?;
		let rows: HashMap<_, _> = select!(&mut tx, joel.id(), peggy.id());
		assert!(!rows.contains_key(&joel.id()));
		assert!(rows.contains_key(&peggy.id()));
		assert_eq!(rows.len(), 1);

		Ok(())
	}

	#[tokio::test]
	#[traced_test]
	async fn retrieve() -> DynResult<()>
	{
		let pool = connect_pg();
		let mut tx = pool.begin().await?;
		let (joel, peggy) = setup(&mut tx).await?;
		tx.commit().await?;

		#[rustfmt::skip]
		let peggy_row = PgUser::retrieve(&pool, peggy.id().into()).await.map(|mut v| v.remove(0))?;
		let joel_row = PgUser::retrieve(&pool, joel.id().into()).await.map(|mut v| v.remove(0))?;

		assert_eq!(joel.employee().map(|e| e.id), joel_row.employee().map(|e| e.id));
		assert_eq!(joel.id(), joel_row.id());
		assert_eq!(joel.password(), joel_row.password());
		assert_eq!(joel.password_expires(), joel_row.password_expires());
		assert_eq!(joel.role().id(), joel_row.role().id());
		assert_eq!(joel.username(), joel_row.username());
		assert_eq!(peggy.employee().map(|e| e.id), peggy_row.employee().map(|e| e.id));
		assert_eq!(peggy.id(), peggy_row.id());
		assert_eq!(peggy.password(), peggy_row.password());
		assert_eq!(peggy.password_expires(), peggy_row.password_expires());
		assert_eq!(peggy.role().id(), peggy_row.role().id());
		assert_eq!(peggy.username(), peggy_row.username());
		assert_str_eq!(joel.password(), joel_row.password());
		assert_str_eq!(joel.username(), joel_row.username());
		assert_str_eq!(peggy.password(), peggy_row.password());
		assert_str_eq!(peggy.username(), peggy_row.username());

		sqlx::query!("DELETE FROM users WHERE id IN ($1, $2);", joel.id(), peggy.id())
			.execute(&pool)
			.await?;

		sqlx::query!(
			"DELETE FROM roles WHERE id IN ($1, $2);",
			joel.role().id(),
			peggy.role().id()
		)
		.execute(&pool)
		.await?;

		Ok(())
	}

	#[tokio::test]
	#[traced_test]
	async fn update() -> DynResult<()>
	{
		let pool = connect_pg();
		let mut tx = pool.begin().await?;
		let (mut joel, peggy) = setup(&mut tx).await?;

		joel = {
			let joel_emp =
				PgEmployee::create(&mut tx, "Joel".into(), "Hired".into(), "Intern".into()).await?;

			let intern = PgRole::create(
				&mut tx,
				"intern".into(),
				Duration::from_secs(rand::random::<u32>().into()).into(),
			)
			.await?;

			User::new(
				joel_emp.into(),
				joel.id(),
				different_string(joel.password()),
				intern,
				different_string(joel.username()),
			)
			.map(|u| u.pg_sanitize())?
		};

		PgUser::update(&mut tx, [&joel].into_iter()).await?;
		let rows: HashMap<_, _> = select!(&mut tx, joel.id(), peggy.id());
		let joel_row = rows
			.get(&joel.id())
			.ok_or_else(|| "The `joel` row does not exist in the database".to_owned())?;

		assert_eq!(joel.employee().map(|e| e.id), joel_row.employee_id);
		assert_eq!(joel.id(), joel_row.id);
		assert_eq!(joel.password(), joel_row.password);
		assert_eq!(joel.password_expires().map(|d| d.naive_utc()), joel_row.password_expires);
		assert_eq!(joel.role().id(), joel_row.role_id);
		assert_eq!(joel.username(), joel_row.username);
		assert_str_eq!(joel.password(), joel_row.password);
		assert_str_eq!(joel.username(), joel_row.username);
		assert_eq!(rows.len(), 2);

		Ok(())
	}
}
