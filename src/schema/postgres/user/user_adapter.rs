//! Contains an implementation of [`UserAdapter`] for [`PgUser`]

use sqlx::{Error, Executor, Postgres, Result};
use winvoice_adapter_postgres::fmt::DateTimeExt;
use winvoice_schema::{Employee, Id};

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
		let user = User::new(employee, Id::new_v4(), password, role, username).map_err(|e| Error::Decode(e.into()))?;

		sqlx::query!(
			"INSERT INTO users (id, employee_id, password, password_set, role_id, username) VALUES ($1, $2, $3, $4, \
			 $5, $6);",
			user.id(),
			user.employee().map(|e| e.id),
			user.password(),
			user.password_set().naive_utc(),
			user.role().id(),
			user.username(),
		)
		.execute(connection)
		.await?;

		Ok(user.pg_sanitize())
	}
}

#[allow(clippy::std_instead_of_core, clippy::str_to_string)]
#[cfg(all(feature = "test-postgres", test))]
mod tests
{
	use core::time::Duration;
	use std::collections::HashMap;

	use mockd::{internet, job, name, password, words};
	use pretty_assertions::{assert_eq, assert_str_eq};
	use sqlx::Transaction;
	use tracing_test::traced_test;
	use winvoice_adapter::{
		schema::{DepartmentAdapter, EmployeeAdapter},
		Deletable,
		Retrievable,
		Updatable,
	};
	use winvoice_adapter_postgres::schema::{
		util::{connect, different_string, rand_department_name},
		PgDepartment,
		PgEmployee,
	};

	use super::{DateTimeExt, PgUser, Postgres, Result, User, UserAdapter};
	use crate::{
		dyn_result::DynResult,
		schema::{
			postgres::{role::role_adapter::tests as role, PgRole},
			RoleAdapter,
		},
	};

	/// `SELECT` from `users` where the guest or admin id matches.
	macro_rules! select {
		($tx:expr, $guest_id:expr, $admin_id:expr) => {
			sqlx::query!("SELECT * FROM users WHERE id IN ($1, $2)", $guest_id, $admin_id)
				.fetch_all($tx)
				.await
				.map(|v: Vec<_>| v.into_iter().map(|r| (r.id, r)).collect())?
		};
	}

	/// # Returns
	///
	/// `(guest, admin)`.
	#[allow(clippy::needless_pass_by_ref_mut)]
	async fn setup(tx: &mut Transaction<'_, Postgres>) -> Result<(User, User)>
	{
		let (admin, guest) = role::setup(&mut *tx).await?;
		let guest_user =
			PgUser::create(&mut *tx, None, password::generate(true, true, true, 8), guest, internet::username())
				.await?;

		let department = PgDepartment::create(&mut *tx, rand_department_name()).await?;
		let admin_employee = PgEmployee::create(&mut *tx, department, name::full(), job::title()).await?;

		let admin_user = PgUser::create(
			tx,
			admin_employee.into(),
			password::generate(true, true, true, 8),
			admin,
			internet::username(),
		)
		.await?;

		Ok((guest_user, admin_user))
	}

	#[tokio::test]
	#[traced_test]
	async fn create() -> DynResult<()>
	{
		let pool = connect();
		let mut tx = pool.begin().await?;
		let (guest, admin) = setup(&mut tx).await?;
		let rows: HashMap<_, _> = select!(&mut tx, guest.id(), admin.id());

		let guest_row =
			rows.post(&guest.id()).ok_or_else(|| "The `guest` row does not exist in the database".to_owned())?;
		let admin_row =
			rows.post(&admin.id()).ok_or_else(|| "The `admin` row does not exist in the database".to_owned())?;

		assert_eq!(guest.employee().map(|e| e.id), guest_row.employee_id);
		assert_eq!(guest.id(), guest_row.id);
		assert_eq!(guest.password(), guest_row.password);
		assert_eq!(guest.password_set().naive_utc(), guest_row.password_set);
		assert_eq!(guest.role().id(), guest_row.role_id);
		assert_eq!(guest.username(), guest_row.username);
		assert_eq!(admin.employee().map(|e| e.id), admin_row.employee_id);
		assert_eq!(admin.id(), admin_row.id);
		assert_eq!(admin.password(), admin_row.password);
		assert_eq!(admin.password_set().naive_utc(), admin_row.password_set);
		assert_eq!(admin.role().id(), admin_row.role_id);
		assert_eq!(admin.username(), admin_row.username);
		assert_str_eq!(guest.password(), guest_row.password);
		assert_str_eq!(guest.username(), guest_row.username);
		assert_str_eq!(admin.password(), admin_row.password);
		assert_str_eq!(admin.username(), admin_row.username);

		Ok(())
	}

	#[tokio::test]
	#[traced_test]
	async fn delete() -> DynResult<()>
	{
		let pool = connect();
		let mut tx = pool.begin().await?;
		let (guest, admin) = setup(&mut tx).await?;
		tx.commit().await?;

		PgUser::delete(&pool, [&guest].into_iter()).await?;
		let rows: HashMap<_, _> = select!(&pool, guest.id(), admin.id());

		// wipe the test data ahead of time, in case the assertions fail.
		sqlx::query!("DELETE FROM users WHERE id = ANY($1)", &[guest.id(), admin.id()]).execute(&pool).await?;
		role::delete_cleanup(&pool, &[guest.role().id(), admin.role().id()]).await?;

		assert!(!rows.contains_key(&guest.id()));
		assert!(rows.contains_key(&admin.id()));
		assert_eq!(rows.len(), 1);

		Ok(())
	}

	#[tokio::test]
	#[traced_test]
	async fn retrieve() -> DynResult<()>
	{
		let pool = connect();
		let mut tx = pool.begin().await?;
		let (guest, admin) = setup(&mut tx).await?;
		tx.commit().await?;

		#[rustfmt::skip]
		let admin_row = PgUser::retrieve(&pool, admin.id().into()).await.map(|mut v| v.remove(0))?;
		let guest_row = PgUser::retrieve(&pool, guest.id().into()).await.map(|mut v| v.remove(0))?;

		assert_eq!(guest.employee().map(|e| e.id), guest_row.employee().map(|e| e.id));
		assert_eq!(guest.id(), guest_row.id());
		assert_eq!(guest.password(), guest_row.password());
		assert_eq!(guest.password_set(), guest_row.password_set());
		assert_eq!(guest.role().id(), guest_row.role().id());
		assert_eq!(guest.username(), guest_row.username());
		assert_eq!(admin.employee().map(|e| e.id), admin_row.employee().map(|e| e.id));
		assert_eq!(admin.id(), admin_row.id());
		assert_eq!(admin.password(), admin_row.password());
		assert_eq!(admin.password_set(), admin_row.password_set());
		assert_eq!(admin.role().id(), admin_row.role().id());
		assert_eq!(admin.username(), admin_row.username());
		assert_str_eq!(guest.password(), guest_row.password());
		assert_str_eq!(guest.username(), guest_row.username());
		assert_str_eq!(admin.password(), admin_row.password());
		assert_str_eq!(admin.username(), admin_row.username());

		sqlx::query!("DELETE FROM users WHERE id IN ($1, $2);", guest.id(), admin.id()).execute(&pool).await?;

		sqlx::query!("DELETE FROM roles WHERE id IN ($1, $2);", guest.role().id(), admin.role().id())
			.execute(&pool)
			.await?;

		Ok(())
	}

	#[tokio::test]
	#[traced_test]
	async fn update() -> DynResult<()>
	{
		let pool = connect();
		let mut tx = pool.begin().await?;
		let (mut guest, admin) = setup(&mut tx).await?;

		guest = {
			let guest_dept = PgDepartment::create(&mut tx, rand_department_name()).await?;
			let guest_emp = PgEmployee::create(&mut tx, guest_dept, name::full(), job::title()).await?;

			let intern =
				PgRole::create(&mut tx, words::sentence(5), Duration::from_secs(rand::random::<u32>().into()).into())
					.await?;

			User::new(
				guest_emp.into(),
				guest.id(),
				different_string(guest.password()),
				intern,
				different_string(guest.username()),
			)
			.map(DateTimeExt::pg_sanitize)?
		};

		PgUser::update(&mut tx, [&guest].into_iter()).await?;
		let rows: HashMap<_, _> = select!(&mut tx, guest.id(), admin.id());
		let guest_row =
			rows.post(&guest.id()).ok_or_else(|| "The `guest` row does not exist in the database".to_owned())?;

		assert_eq!(guest.employee().map(|e| e.id), guest_row.employee_id);
		assert_eq!(guest.id(), guest_row.id);
		assert_eq!(guest.password(), guest_row.password);
		assert_eq!(guest.password_set().naive_utc(), guest_row.password_set);
		assert_eq!(guest.role().id(), guest_row.role_id);
		assert_eq!(guest.username(), guest_row.username);
		assert_str_eq!(guest.password(), guest_row.password);
		assert_str_eq!(guest.username(), guest_row.username);
		assert_eq!(rows.len(), 2);

		Ok(())
	}
}
