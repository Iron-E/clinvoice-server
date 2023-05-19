//! Contains data regarding what is stored when a user logs in.

use winvoice_schema::chrono::{DateTime, Local};

/// Represents a user who has successfully logged in, and may *stay* logged in.
pub(super) struct Session
{
	/// The [`DateTime`] that this session was created. Stored for the purposes of ensuring expiry
	/// is done on time.
	pub(super) date: DateTime<Local>,

	/// The username of the user who has logged in.
	pub(super) username: String,

	/// The password of the user who has logged in.
	pub(super) password: String,
}
