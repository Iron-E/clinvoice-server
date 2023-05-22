//! Contains the [`Hash`] implementation for [`Session`]s.

use core::hash::{Hash, Hasher};

use sqlx::Database;

use super::Session;

impl<Db> Hash for Session<Db>
where
	Db: Database,
{
	fn hash<H>(&self, state: &mut H)
	where
		H: Hasher,
	{
		self.date.hash(state);
		self.password.hash(state);
		self.username.hash(state);
	}
}
