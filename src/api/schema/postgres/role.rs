//! Contains a  [`Role`](crate::api::schema::Role) for the [`Postgres`](sqlx::Postgres) database.

use sqlx::{postgres::PgRow, Result, Row};
use winvoice_adapter_postgres::schema::util::duration_from;
use winvoice_schema::Id;

use crate::api::schema::{columns::RoleColumns, Role};

mod deletable;
mod retrievable;
mod role_adapter;
mod updatable;

/// A [`Role`](crate::api::schema::Role) which has specialized implementations for the
/// [`Postgres`](sqlx::Postgres) database.
pub struct PgRole;

impl PgRole
{
	pub fn row_to_view(columns: &RoleColumns, row: &PgRow) -> Result<Role>
	{
		let id = row.try_get::<Id, _>(columns.id)?;
		let name = row.try_get::<String, _>(columns.name)?;
		let password_ttl = row
			.try_get::<Option<_>, _>(columns.password_ttl)
			.and_then(|ttl| ttl.map(duration_from).transpose())?;

		Ok(Role { id, name, password_ttl })
	}
}
