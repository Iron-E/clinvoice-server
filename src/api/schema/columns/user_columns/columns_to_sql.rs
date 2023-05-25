use core::fmt::Display;

use sqlx::{Database, QueryBuilder};

use super::TimesheetColumns;
use crate::fmt::{ColumnsToSql, QueryBuilderExt};

impl<Column> ColumnsToSql for TimesheetColumns<Column>
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
			.push(self.job_id)
			.push(self.time_begin)
			.push(self.time_end)
			.push(self.work_notes);
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
			.push_equal(self.job_id, values_columns.job_id)
			.push(',')
			.push_equal(self.time_begin, values_columns.time_begin)
			.push(',')
			.push_equal(self.time_end, values_columns.time_end)
			.push(',')
			.push_equal(self.work_notes, values_columns.work_notes);
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
