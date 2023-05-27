//! Contains information about the [`Role`] of a [`User`](super::User).

use core::time::Duration;

use serde::{Deserialize, Serialize};
use winvoice_schema::Id;

/// Corresponds to the `users` table in the [`winvoice-server`](crate) database.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[cfg_attr(feature = "bin", derive(sqlx::FromRow))]
pub struct Role
{
	/// The unique identity of the [`Role`].
	id: Id,

	/// The name of the [`Role`].
	name: String,

	/// How frequent password rotation must occur for [`User`](super::User) with this [`Role`].
	///
	/// [`None`] indicates that the password lasts forever.
	password_ttl: Option<Duration>,
}

impl Role
{
	/// Create a new [`Role`].
	pub fn new(id: Id, name: String, password_ttl: Option<Duration>) -> Self
	{
		Self { id, name, password_ttl }
	}

	/// The unique identity of the [`Role`].
	pub fn id(&self) -> i64
	{
		self.id
	}

	/// The name of the [`Role`].
	pub fn name(&self) -> &str
	{
		self.name.as_ref()
	}

	/// How frequent password rotation must occur for [`User`](super::User) with this [`Role`].
	///
	/// [`None`] indicates that the password lasts forever.
	pub fn password_ttl(&self) -> Option<Duration>
	{
		self.password_ttl
	}
}