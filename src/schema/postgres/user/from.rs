//! Implementations of [`Into`] for [`PgUser`].

use sqlx::{Postgres, QueryBuilder};
use winvoice_adapter::{
	fmt::{sql, QueryBuilderExt},
	schema::columns::{DepartmentColumns, EmployeeColumns},
};

use super::PgUser;
use crate::schema::columns::{RoleColumns, UserColumns};

impl<'args> From<PgUser> for QueryBuilder<'args, Postgres>
{
	fn from(_: PgUser) -> Self
	{
		const COLUMNS: UserColumns = UserColumns::default();
		const DEPARTMENT_COLUMNS_UNIQUE: DepartmentColumns = DepartmentColumns::unique();
		const EMPLOYEE_COLUMNS_UNIQUE: EmployeeColumns = EmployeeColumns::unique();
		const ROLE_COLUMNS_UNIQUE: RoleColumns = RoleColumns::unique();

		let columns = COLUMNS.default_scope();
		let department_columns = DepartmentColumns::default().default_scope();
		let employee_columns = EmployeeColumns::default().default_scope();
		let role_columns = RoleColumns::default().default_scope();

		let mut query = QueryBuilder::new(sql::SELECT);
		query
			.push_columns(&columns)
			.push_more_columns(&department_columns.r#as(DEPARTMENT_COLUMNS_UNIQUE))
			.push_more_columns(&employee_columns.r#as(EMPLOYEE_COLUMNS_UNIQUE))
			.push_more_columns(&role_columns.r#as(ROLE_COLUMNS_UNIQUE))
			.push_default_from::<UserColumns>()
			.push(sql::LEFT)
			.push_default_equijoin::<EmployeeColumns, _, _>(employee_columns.id, columns.employee_id)
			.push(sql::LEFT)
			.push_default_equijoin::<DepartmentColumns, _, _>(department_columns.id, employee_columns.department_id)
			.push_default_equijoin::<RoleColumns, _, _>(role_columns.id, columns.role_id);

		query
	}
}
