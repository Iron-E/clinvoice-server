//! Contains a [`Clone`] implementation for [`State`].

use super::{Database, State};

impl<Db> Clone for State<Db>
where
	Db: Database,
{
	fn clone(&self) -> Self
	{
		Self { permissions: self.permissions.clone(), pool: self.pool.clone() }
	}
}
