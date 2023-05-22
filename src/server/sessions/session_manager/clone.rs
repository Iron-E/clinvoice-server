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
			domain: self.domain.clone(),
			refresh_by_id: self.refresh_by_id.clone(),
			refresh_secret: self.refresh_secret.clone(),
			refresh_ttl: self.refresh_ttl,
			session_ttl_core: self.session_ttl_core,
			session_ttl_time: self.session_ttl_time,
			sessions_by_id: self.sessions_by_id.clone(),
		}
	}
}
