//! This module contains the response for a login.

use serde::{Deserialize, Serialize};

/// The login request response.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct WhoAmI
{
	/// The username of the [`User`](crate::schema::User).
	username: String,
}

impl WhoAmI
{
	/// Create a new [`WhoAmI`] response.
	pub const fn new(username: String) -> Self
	{
		Self { username }
	}

	/// The username of the [`User`](crate::schema::User).
	#[allow(dead_code)]
	pub fn username(&self) -> &str
	{
		self.username.as_ref()
	}
}
