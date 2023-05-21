//! Contains data regarding what is stored when a user logs in.

use winvoice_schema::chrono::{DateTime, Utc};

/// Represents a user who has successfully logged in, and may *stay* logged in.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Session
{
	/// The [`DateTime`] that this session was created. Stored for the purposes of ensuring expiry
	/// is done on time.
	date: DateTime<Utc>,

	/// The username of the user who has logged in.
	username: String,

	/// The password of the user who has logged in.
	password: String,
}

impl Session
{
	pub(super) fn new(username: String, password: String) -> Self
	{
		Self { date: Utc::now(), username, password }
	}
}
