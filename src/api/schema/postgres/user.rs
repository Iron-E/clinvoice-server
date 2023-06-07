//! Contains a  [`User`](crate::api::schema::User) for the [`Postgres`](sqlx::Postgres) database.

use sqlx::{Postgres, QueryBuilder};
use winvoice_adapter::{
	fmt::{sql, QueryBuilderExt},
	schema::columns::EmployeeColumns,
};

use crate::api::schema::columns::{RoleColumns, UserColumns};

mod deletable;
mod retrievable;
mod updatable;
mod user_adapter;

/// A [`User`](crate::api::schema::User) which has specialized implementations for the
/// [`Postgres`](sqlx::Postgres) database.
pub struct PgUser;

impl PgUser
{
	pub fn select<'args>() -> QueryBuilder<'args, Postgres>
	{
		const COLUMNS: UserColumns = UserColumns::default();
		const EMPLOYEE_COLUMNS_UNIQUE: EmployeeColumns = EmployeeColumns::unique();
		const ROLE_COLUMNS_UNIQUE: RoleColumns = RoleColumns::unique();

		let columns = COLUMNS.default_scope();
		let employee_columns = EmployeeColumns::default().default_scope();
		let role_columns = RoleColumns::default().default_scope();

		let mut query = QueryBuilder::new(sql::SELECT);
		query
			.push_columns(&columns)
			.push_more_columns(&employee_columns.r#as(EMPLOYEE_COLUMNS_UNIQUE))
			.push_more_columns(&role_columns.r#as(ROLE_COLUMNS_UNIQUE))
			.push_default_from::<UserColumns>()
			.push(sql::LEFT)
			.push_default_equijoin::<EmployeeColumns, _, _>(
				employee_columns.id,
				columns.employee_id,
			)
			.push_default_equijoin::<RoleColumns, _, _>(role_columns.id, columns.role_id);

		query
	}
}
