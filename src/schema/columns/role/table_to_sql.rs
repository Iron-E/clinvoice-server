use winvoice_adapter::fmt::TableToSql;

use super::RoleColumns;

impl TableToSql for RoleColumns
{
	const DEFAULT_ALIAS: char = 'R';
	const TABLE_NAME: &'static str = "roles";
}
