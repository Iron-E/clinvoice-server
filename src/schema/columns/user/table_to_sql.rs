use winvoice_adapter::fmt::TableToSql;

use super::UserColumns;

impl TableToSql for UserColumns
{
	const DEFAULT_ALIAS: char = 'U';
	const TABLE_NAME: &'static str = "users";
}
