//! This module contains the response for a login.

use serde::{Deserialize, Serialize};

use crate::api::{Status, StatusCode, Token};

/// The login request response.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Login
{
	/// The [`Status`] of the login request.
	status: Status,

	/// The unique identifier that can be used to gain access to the server again.
	token: Option<Token>,
}

impl Login
{
	/// Create a new [`Login`] response.
	pub fn new(code: StatusCode, message: Option<String>, token: Option<Token>) -> Self
	{
		Self { status: Status::new(code, message), token }
	}

	/// The [`Status`] of the login request.
	pub const fn status(&self) -> &Status
	{
		&self.status
	}

	pub const fn token(&self) -> Option<Token>
	{
		self.token
	}
}
