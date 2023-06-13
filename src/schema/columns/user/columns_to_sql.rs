use core::fmt::Display;

use sqlx::{Database, QueryBuilder};
use winvoice_adapter::fmt::{ColumnsToSql, QueryBuilderExt};

use super::UserColumns;

impl<Column> ColumnsToSql for UserColumns<Column>
where
	Column: Copy + Display,
{
	fn push_to<Db>(&self, query: &mut QueryBuilder<Db>)
	where
		Db: Database,
	{
		query
			.separated(',')
			.push(self.employee_id)
			.push(self.id)
			.push(self.password)
			.push(self.password_expires)
			.push(self.role_id)
			.push(self.username);
	}

	fn push_set_to<Db, Values>(&self, query: &mut QueryBuilder<Db>, values_alias: Values)
	where
		Db: Database,
		Values: Copy + Display,
	{
		let values_columns = self.scope(values_alias);
		query
			.push_equal(self.employee_id, values_columns.employee_id)
			.push(',')
			.push_equal(self.password, values_columns.password)
			.push(',')
			.push_equal(self.password_expires, values_columns.password_expires)
			.push(',')
			.push_equal(self.role_id, values_columns.role_id)
			.push(',')
			.push_equal(self.username, values_columns.username);
	}

	fn push_update_where_to<Db, Table, Values>(
		&self,
		query: &mut QueryBuilder<Db>,
		table_alias: Table,
		values_alias: Values,
	) where
		Db: Database,
		Table: Copy + Display,
		Values: Copy + Display,
	{
		query.push_equal(self.scope(table_alias).id, self.scope(values_alias).id);
	}
}
