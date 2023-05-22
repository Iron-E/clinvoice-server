//! This module contains the response for a logout.

use serde::{Deserialize, Serialize};

use crate::api::Status;

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
	pub fn new<S>(status: S) -> Self
	where
		S: Into<Status>,
	{
		Self { status: status.into() }
	}

	/// The [`Status`] of the logout request.
	pub const fn status(&self) -> &Status
	{
		&self.status
	}
}
