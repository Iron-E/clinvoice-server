//! Implementation of [`FromRow`] for [`User`].

use sqlx::{FromRow, Result, Row};
use winvoice_adapter::schema::columns::EmployeeColumns;
use winvoice_schema::Id;

use super::User;
use crate::api::schema::columns::{RoleColumns, UserColumns};

const EMPLOYEE_COLUMNS: EmployeeColumns = EmployeeColumns::unique();
const ROLE_COLUMNS: RoleColumns = RoleColumns::unique();
const USER_COLUMNS: UserColumns = UserColumns::default();

#[cfg(feature = "postgres")]
mod postgres
{
	use sqlx::postgres::PgRow;
	use winvoice_adapter_postgres::schema::{util as pg_util, PgEmployee};

	#[allow(clippy::wildcard_imports)]
	use super::*;
	use crate::api::schema::postgres::PgRole;

	impl FromRow<'_, PgRow> for User
	{
		fn from_row(row: &PgRow) -> Result<Self>
		{
			let employee_id: Option<Id> = row.try_get(EMPLOYEE_COLUMNS.id)?;

			Ok(Self {
				employee: employee_id.map(|_| PgEmployee::row_to_view(EMPLOYEE_COLUMNS, row)),
				id: row.try_get(USER_COLUMNS.id)?,
				password: row.try_get(USER_COLUMNS.password)?,
				password_expires: row
					.try_get(USER_COLUMNS.password_expires)
					.map(pg_util::naive_date_opt_to_utc)?,
				role: PgRole::row_to_view(&ROLE_COLUMNS, row)?,
				username: row.try_get(USER_COLUMNS.username)?,
			})
		}
	}
}
