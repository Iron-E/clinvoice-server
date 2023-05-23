//! Contains the [`Debug`] implementation for [`SessionManager`]

use core::{
	any,
	fmt::{Debug, Formatter, Result},
};

use sqlx::Database;

use super::SessionManager;

impl<Db> Debug for SessionManager<Db>
where
	Db: Database,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> Result
	{
		let name = format!("SessionManager<{}>", any::type_name::<Db>());
		f.debug_struct(&name)
			.field("connect_options", &self.connect_options)
			.field("refresh_by_id", &self.refresh_by_id)
			.field("refresh_ttl", &self.refresh_ttl)
			.field("session_ttl", &self.session_idle)
			.field("session_by_id", &self.sessions_by_id)
			.finish_non_exhaustive()
	}
}
