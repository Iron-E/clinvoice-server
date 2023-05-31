//! Contains a [`Clone`] impl for [`DbSessionStore`]

use super::{Database, DbSessionStore};

impl<Db> Clone for DbSessionStore<Db>
where
	Db: Database,
{
	fn clone(&self) -> Self
	{
		Self { pool: self.pool.clone() }
	}
}
