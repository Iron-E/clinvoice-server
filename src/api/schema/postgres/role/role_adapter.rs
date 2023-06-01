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
	use sqlx::Transaction;
	use winvoice_adapter::Retrievable;
	use winvoice_adapter_postgres::schema::util::duration_from;
	use winvoice_schema::Id;

	use super::{Duration, PgRole, Postgres, RoleAdapter};
	use crate::api::schema::Role;

	/// `SECONDS_PER_MINUTE * MINUTES_PER_SECOND * HOURS_PER_DAY * DAYS_PER_MONTH`
	const SECONDS_PER_MONTH: u64 = 60 * 60 * 24 * 30;

	/// Returns `(admin, guest)`.
	pub(in crate::api::schema::postgres) async fn insert_mock_roles(
		tx: &mut Transaction<'_, Postgres>,
		offset: u8,
	) -> (Role, Role)
	{
		(
			PgRole::create(
				&mut *tx,
				format!("admin{offset}"),
				Duration::from_secs(SECONDS_PER_MONTH).into(),
			)
			.await
			.unwrap(),
			PgRole::create(
				&mut *tx,
				format!("guest{offset}"),
				Duration::from_secs(SECONDS_PER_MONTH * 3).into(),
			)
			.await
			.unwrap(),
		)
	}

	#[tokio::test]
	async fn create()
	{
		let pool = crate::utils::connect_pg();
		let mut tx = pool.begin().await.unwrap();
		let (admin, guest) = insert_mock_roles(&mut tx, 1).await;

		let rows: HashMap<Id, _> =
			sqlx::query!("SELECT * FROM roles WHERE id IN ($1, $2)", admin.id(), guest.id())
				.fetch_all(&mut tx)
				.await
				.map(|v: Vec<_>| v.into_iter().map(|r| (r.id, r)).collect())
				.unwrap();

		assert_eq!(rows.len(), 2);

		let admin_row = rows.get(&admin.id()).unwrap();
		assert_eq!(admin.id(), admin_row.id);
		assert_eq!(admin.name(), admin_row.name);
		assert_eq!(
			admin.password_ttl(),
			admin_row.password_ttl.clone().map(|ttl| duration_from(ttl).unwrap())
		);

		let guest_row = rows.get(&guest.id()).unwrap();
		assert_eq!(guest.id(), guest_row.id);
		assert_eq!(guest.name(), guest_row.name);
		assert_eq!(
			guest.password_ttl(),
			guest_row.password_ttl.clone().map(|ttl| duration_from(ttl).unwrap())
		);
	}

	#[tokio::test]
	async fn retrieve()
	{
		let pool = crate::utils::connect_pg();
		let mut tx = pool.begin().await.unwrap();
		let (admin, guest) = insert_mock_roles(&mut tx, 2).await;

		tx.commit().await.unwrap();

		let admin_row =
			PgRole::retrieve(&pool, admin.id().into()).await.map(|mut v| v.remove(0)).unwrap();
		assert_eq!(admin.id(), admin_row.id());
		assert_eq!(admin.name(), admin_row.name());
		assert_eq!(admin.password_ttl(), admin_row.password_ttl());

		let guest_row =
			PgRole::retrieve(&pool, guest.id().into()).await.map(|mut v| v.remove(0)).unwrap();
		assert_eq!(guest.id(), guest_row.id());
		assert_eq!(guest.name(), guest_row.name());
		assert_eq!(guest.password_ttl(), guest_row.password_ttl());

		// cleanup
		sqlx::query!("DELETE FROM roles WHERE id IN ($1, $2);", admin.id(), guest.id())
			.execute(&pool)
			.await
			.unwrap();
	}
}
