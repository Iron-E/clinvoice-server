use winvoice_adapter::fmt::TableToSql;

use super::UserColumns;

impl<T> TableToSql for UserColumns<T>
{
	const DEFAULT_ALIAS: char = 'U';
	const TABLE_NAME: &'static str = "users";
}
