//! Contains an implementation of [`RoleAdapter`] for [`PgRole`]

use core::time::Duration;

use sqlx::{Executor, Postgres, Result};
use winvoice_schema::Id;

use super::PgRole;
use crate::schema::{Role, RoleAdapter};

#[async_trait::async_trait]
impl RoleAdapter for PgRole
{
	#[tracing::instrument(level = "trace", skip(connection), err)]
	async fn create<'connection, Conn>(
		connection: Conn,
		name: String,
		password_ttl: Option<Duration>,
	) -> Result<Role>
	where
		Conn: Executor<'connection, Database = Postgres>,
	{
		let id = Id::new_v4();
		sqlx::query!(
			"INSERT INTO roles (id, name, password_ttl) VALUES ($1, $2, $3);",
			id,
			name,
			password_ttl as _,
		)
		.execute(connection)
		.await?;

		Ok(Role::new(id, name, password_ttl))
	}
}

#[cfg(all(feature = "test-postgres", test))]
pub(in crate::schema::postgres) mod tests
{
	use std::collections::HashMap;

	use mockd::words;
	use pretty_assertions::{assert_eq, assert_str_eq};
	use sqlx::Transaction;
	use tracing_test::traced_test;
	use winvoice_adapter::{Deletable, Retrievable, Updatable};
	use winvoice_adapter_postgres::schema::util::{connect, different_string, duration_from};

	use super::{Duration, PgRole, Postgres, Result, RoleAdapter};
	use crate::{dyn_result::DynResult, schema::Role};

	/// `SECONDS_PER_MINUTE * MINUTES_PER_SECOND * HOURS_PER_DAY * DAYS_PER_MONTH`
	const SECONDS_PER_MONTH: u64 = 60 * 60 * 24 * 30;

	/// `SELECT` from `roles` where the admin or guest id matches.
	macro_rules! select {
		($tx:expr, $admin_id:expr, $guest_id:expr) => {
			sqlx::query!("SELECT * FROM roles WHERE id IN ($1, $2)", $admin_id, $guest_id)
				.fetch_all($tx)
				.await
				.map(|v: Vec<_>| v.into_iter().map(|r| (r.id, r)).collect())?
		};
	}

	/// # Returns
	///
	/// `(admin, guest)`.
	pub async fn setup(tx: &mut Transaction<'_, Postgres>) -> Result<(Role, Role)>
	{
		let admin = PgRole::create(
			&mut *tx,
			words::sentence(4),
			Duration::from_secs(SECONDS_PER_MONTH).into(),
		)
		.await?;

		let guest = PgRole::create(&mut *tx, words::sentence(4), None).await?;

		Ok((admin, guest))
	}

	#[tokio::test]
	#[traced_test]
	async fn create() -> DynResult<()>
	{
		let pool = connect();
		let mut tx = pool.begin().await?;
		let (admin, guest) = setup(&mut tx).await?;
		let rows: HashMap<_, _> = select!(&mut tx, admin.id(), guest.id());

		let admin_row = rows
			.get(&admin.id())
			.ok_or_else(|| "The `admin` row does not exist in the database".to_owned())?;
		let guest_row = rows
			.get(&guest.id())
			.ok_or_else(|| "The `guest` row does not exist in the database".to_owned())?;

		let admin_row_password_ttl =
			admin_row.password_ttl.clone().map(duration_from).transpose()?;
		let guest_row_password_ttl =
			guest_row.password_ttl.clone().map(duration_from).transpose()?;

		assert_eq!(admin.id(), admin_row.id);
		assert_eq!(admin.password_ttl(), admin_row_password_ttl);
		assert_eq!(guest.id(), guest_row.id);
		assert_eq!(guest.password_ttl(), guest_row_password_ttl);
		assert_str_eq!(admin.name(), admin_row.name);
		assert_str_eq!(guest.name(), guest_row.name);

		Ok(())
	}

	#[tokio::test]
	#[traced_test]
	async fn delete() -> DynResult<()>
	{
		let pool = connect();
		let mut tx = pool.begin().await?;
		let (admin, guest) = setup(&mut tx).await?;

		PgRole::delete(&mut tx, [&admin].into_iter()).await?;
		let rows: HashMap<_, _> = select!(&mut tx, admin.id(), guest.id());
		assert!(!rows.contains_key(&admin.id()));
		assert!(rows.contains_key(&guest.id()));
		assert_eq!(rows.len(), 1);

		Ok(())
	}

	#[tokio::test]
	#[traced_test]
	async fn retrieve() -> DynResult<()>
	{
		let pool = connect();
		let mut tx = pool.begin().await?;
		let (admin, guest) = setup(&mut tx).await?;

		tx.commit().await?;

		#[rustfmt::skip]
		let admin_row = PgRole::retrieve(&pool, admin.id().into()).await.map(|mut v| v.remove(0))?;
		assert_eq!(admin.id(), admin_row.id());
		assert_str_eq!(admin.name(), admin_row.name());
		assert_eq!(admin.password_ttl(), admin_row.password_ttl());

		#[rustfmt::skip]
		let guest_row = PgRole::retrieve(&pool, guest.id().into()).await.map(|mut v| v.remove(0))?;
		assert_eq!(guest.id(), guest_row.id());
		assert_str_eq!(guest.name(), guest_row.name());
		assert_eq!(guest.password_ttl(), guest_row.password_ttl());

		sqlx::query!("DELETE FROM roles WHERE id IN ($1, $2);", admin.id(), guest.id())
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
		let (mut admin, guest) = setup(&mut tx).await?;
		admin = Role::new(
			admin.id(),
			different_string(admin.name()),
			Duration::from_secs(rand::random::<u32>().into()).into(),
		);

		PgRole::update(&mut tx, [&admin].into_iter()).await?;
		let rows: HashMap<_, _> = select!(&mut tx, admin.id(), guest.id());
		let admin_row = rows
			.get(&admin.id())
			.ok_or_else(|| "The `admin` row does not exist in the database".to_owned())?;
		let admin_row_password_ttl =
			admin_row.password_ttl.clone().map(duration_from).transpose()?;

		assert_eq!(admin.id(), admin_row.id);
		assert_str_eq!(admin.name(), admin_row.name);
		assert_eq!(admin.password_ttl(), admin_row_password_ttl);
		assert_eq!(rows.len(), 2);

		Ok(())
	}
}
