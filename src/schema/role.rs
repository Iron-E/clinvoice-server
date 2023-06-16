//! Contains information about the [`Role`] of a [`User`](super::User).

#![cfg_attr(feature = "bin", allow(clippy::std_instead_of_core))]

use core::time::Duration;

use serde::{Deserialize, Serialize};
use winvoice_schema::Id;

/// Corresponds to the `role` table in the database.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
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
	pub const fn new(id: Id, name: String, password_ttl: Option<Duration>) -> Self
	{
		Self { id, name, password_ttl }
	}

	/// The unique identity of the [`Role`].
	pub const fn id(&self) -> Id
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
	pub const fn password_ttl(&self) -> Option<Duration>
	{
		self.password_ttl
	}
}
