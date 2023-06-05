//! This module contains the response for a login.

mod as_ref;

use serde::{Deserialize, Serialize};

use crate::api::Status;

/// The login request response.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Login
{
	/// The [`Status`] of the login request.
	status: Status,
}

impl Login
{
	/// Create a new [`Login`] response.
	pub const fn new(status: Status) -> Self
	{
		Self { status }
	}

	/// The [`Status`] of the login request.
	pub const fn status(&self) -> &Status
	{
		&self.status
	}
}
