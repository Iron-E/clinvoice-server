//! This module contains the response for a login.

use serde::{Deserialize, Serialize};

use crate::api::{Status, StatusCode};

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
	pub fn new(code: StatusCode, message: Option<String>) -> Self
	{
		Self { status: Status::new(code, message) }
	}

	/// The [`Status`] of the login request.
	pub const fn status(&self) -> &Status
	{
		&self.status
	}
}
