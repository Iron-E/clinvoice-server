//! This module contains the response for a logout.

mod as_ref;

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
	pub const fn new(status: Status) -> Self
	{
		Self { status }
	}

	/// The [`Status`] of the logout request.
	#[allow(dead_code)]
	pub const fn status(&self) -> &Status
	{
		&self.status
	}
}
