//! Contains the [`PartialOrd`] implementation for [`Session`]s.

use core::cmp::Ordering;

use sqlx::Database;

use super::Session;

impl<Db> PartialOrd for Session<Db>
where
	Db: Database,
{
	fn partial_cmp(&self, other: &Self) -> Option<Ordering>
	{
		self.date.partial_cmp(&other.date).or_else(|| {
			self.password
				.partial_cmp(&other.password)
				.or_else(|| self.username.partial_cmp(&other.username))
		})
	}
}
