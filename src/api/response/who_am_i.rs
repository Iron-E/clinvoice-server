//! This module contains the response for a login.

use serde::{Deserialize, Serialize};
use crate::schema::User;

/// The login request response.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct WhoAmI
{
	/// The username of the [`User`](crate::schema::User).
	user: User,
}

impl WhoAmI
{
	/// Create a new [`WhoAmI`] response.
	pub const fn new(user: User) -> Self
	{
		Self { user }
	}

	/// The username of the [`User`](crate::schema::User).
	#[allow(dead_code)]
	pub fn user(&self) -> &User
	{
		&self.user
	}
}
