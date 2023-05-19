use sqlx::{Connection, Database};

use super::SessionManager;

impl<Db> Clone for SessionManager<Db>
where
	Db: Database,
	<Db::Connection as Connection>::Options: Clone,
{
	fn clone(&self) -> Self
	{
		Self {
			connect_options: self.connect_options.clone(),
			connections: self.connections.clone(),
			idle_timeout: self.idle_timeout,
			session_expire: self.session_expire,
			sessions: self.sessions.clone(),
		}
	}
}
