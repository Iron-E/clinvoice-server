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
pub(super) mod tests
{
	use std::collections::HashMap;

	use pretty_assertions::assert_eq;
	use sqlx::{Error, PgPool, Result, Transaction};
	use winvoice_adapter::{Deletable, Retrievable, Updatable};
	use winvoice_adapter_postgres::schema::util::duration_from;
	use winvoice_schema::Id;

	use super::{Duration, PgRole, Postgres, RoleAdapter};
	use crate::{api::schema::Role, dyn_result::DynResult, utils::connect_pg};

	/// `SECONDS_PER_MINUTE * MINUTES_PER_SECOND * HOURS_PER_DAY * DAYS_PER_MONTH`
	const SECONDS_PER_MONTH: u64 = 60 * 60 * 24 * 30;

	/// `SELECT` from `roles` where the admin or guest id matches.
	macro_rules! select {
		($tx:expr, $admin_id:expr, $guest_id:expr) => {
			sqlx::query!("SELECT * FROM roles WHERE id IN ($1, $2)", $admin_id, $guest_id)
				.fetch_all($tx)
				.await
				.map(|v: Vec<_>| v.into_iter().map(|r| (r.id, r)).collect())?;
		};
	}

	/// # Returns
	///
	/// `(admin, guest)`.
	pub(in crate::api::schema::postgres) async fn setup(
		tx: &mut Transaction<'_, Postgres>,
	) -> Result<(Role, Role)>
	{
		let admin = PgRole::create(
			&mut *tx,
			format!("admin{}", rand::random::<[char; 8]>().into_iter().collect::<String>()),
			Duration::from_secs(SECONDS_PER_MONTH).into(),
		)
		.await?;

		let guest = PgRole::create(
			&mut *tx,
			format!("guest{}", rand::random::<[char; 8]>().into_iter().collect::<String>()),
			Duration::from_secs(SECONDS_PER_MONTH * 3).into(),
		)
		.await?;

		Ok((admin, guest))
	}

	/// Cleans up the [`setup`]
	pub(in crate::api::schema::postgres) async fn tear_down(
		pool: &PgPool,
		admin_id: Id,
		guest_id: Id,
	) -> Result<()>
	{
		sqlx::query!("DELETE FROM roles WHERE id IN ($1, $2);", admin_id, guest_id)
			.execute(pool)
			.await?;

		Ok(())
	}

	#[tokio::test]
	async fn create() -> DynResult<()>
	{
		let pool = connect_pg();
		let mut tx = pool.begin().await?;
		let (admin, guest) = setup(&mut tx).await?;

		let rows: HashMap<_, _> = select!(&mut tx, admin.id(), guest.id());

		assert_eq!(rows.len(), 2);

		let admin_row = rows
			.get(&admin.id())
			.ok_or_else(|| "The `admin` row does not exist in the database".to_owned())?;
		let admin_row_password_ttl =
			admin_row.password_ttl.clone().map(duration_from).transpose()?;
		assert_eq!(admin.id(), admin_row.id);
		assert_eq!(admin.name(), admin_row.name);
		assert_eq!(admin.password_ttl(), admin_row_password_ttl);

		let guest_row = rows
			.get(&guest.id())
			.ok_or_else(|| "The `guest` row does not exist in the database".to_owned())?;
		let guest_row_password_ttl =
			guest_row.password_ttl.clone().map(duration_from).transpose()?;
		assert_eq!(guest.id(), guest_row.id);
		assert_eq!(guest.name(), guest_row.name);
		assert_eq!(guest.password_ttl(), guest_row_password_ttl);

		Ok(())
	}

	#[tokio::test]
	async fn delete() -> DynResult<()>
	{
		let pool = connect_pg();
		let mut tx = pool.begin().await?;
		let (admin, guest) = setup(&mut tx).await?;

		PgRole::delete(&mut tx, [&admin].into_iter()).await?;

		let rows: HashMap<_, _> = select!(&mut tx, admin.id(), guest.id());
		assert_eq!(rows.len(), 1);
		assert!(!rows.contains_key(&admin.id()));
		Ok(())
	}

	#[tokio::test]
	async fn retrieve() -> DynResult<()>
	{
		let pool = connect_pg();
		let mut tx = pool.begin().await?;
		let (admin, guest) = setup(&mut tx).await?;

		tx.commit().await?;

		#[rustfmt::skip]
		let admin_row = PgRole::retrieve(&pool, admin.id().into()).await.map(|mut v| v.remove(0))?;
		assert_eq!(admin.id(), admin_row.id());
		assert_eq!(admin.name(), admin_row.name());
		assert_eq!(admin.password_ttl(), admin_row.password_ttl());

		#[rustfmt::skip]
		let guest_row = PgRole::retrieve(&pool, guest.id().into()).await.map(|mut v| v.remove(0))?;
		assert_eq!(guest.id(), guest_row.id());
		assert_eq!(guest.name(), guest_row.name());
		assert_eq!(guest.password_ttl(), guest_row.password_ttl());

		tear_down(&pool, admin.id(), guest.id()).await?;
		Ok(())
	}

	#[tokio::test]
	async fn update() -> DynResult<()>
	{
		todo!()
	}
}
