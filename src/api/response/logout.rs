//! This module contains the response for a logout.

use serde::{Deserialize, Serialize};

use crate::api::{Status, StatusCode};

/// The logout request response.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Logout
{
	/// The [`Status`] of the logout request.
	status: Status,
}

impl Logout
{
	/// Create a new [`Logout`] response.
	pub fn new(code: StatusCode, message: Option<String>) -> Self
	{
		Self { status: Status::new(code, message) }
	}

	/// The [`Status`] of the logout request.
	pub const fn status(&self) -> &Status
	{
		&self.status
	}
}
