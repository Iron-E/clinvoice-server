//! Contains a [`Clone`] implementation for [`State`].

use super::{Database, ServerState};

impl<Db> Clone for ServerState<Db>
where
	Db: Database,
{
	fn clone(&self) -> Self
	{
		Self { permissions: self.permissions.clone(), pool: self.pool.clone() }
	}
}
