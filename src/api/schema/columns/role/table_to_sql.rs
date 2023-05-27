use winvoice_adapter::fmt::TableToSql;

use super::RoleColumns;

impl<T> TableToSql for RoleColumns<T>
{
	const DEFAULT_ALIAS: char = 'R';
	const TABLE_NAME: &'static str = "roles";
}
