//! This module contains the response for a login.

mod as_ref;
mod from;

use serde::{Deserialize, Serialize};

use crate::{api::Status, schema::User};

/// The login request response.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Login
{
	/// The [`Status`] of the login response.
	status: Status,

	/// The [`User`] being logged in.
	user: Option<User>,
}

impl Login
{
	/// Create a new [`Login`] response.
	pub const fn new(status: Status, user: Option<User>) -> Self
	{
		Self { status, user }
	}

	/// The [`Status`] of the login response.
	#[allow(dead_code)]
	pub const fn status(&self) -> &Status
	{
		&self.status
	}

	/// The [`User`] of the login resposne.
	#[allow(dead_code)]
	pub const fn user(&self) -> Option<&User>
	{
		self.user.as_ref()
	}
}
