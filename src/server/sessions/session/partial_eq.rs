//! Contains the [`PartialEq`] implementation for [`Session`]s.

use sqlx::Database;

use super::Session;

impl<Db> PartialEq for Session<Db>
where
	Db: Database,
{
	fn eq(&self, other: &Self) -> bool
	{
		self.date.eq(&other.date) &&
			self.password.eq(&other.password) &&
			self.username.eq(&other.username)
	}
}
