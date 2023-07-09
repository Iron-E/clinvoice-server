//! This module contains the response for a login.

mod as_ref;

use serde::{Deserialize, Serialize};

use crate::api::Status;

/// The DELETE & [`winvoice_adapter::Deletable::delete`] request response.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Delete
{
	/// The [`Status`] of the login request.
	status: Status,
}

impl Delete
{
	/// The [`Status`] of the login request.
	#[allow(dead_code)]
	pub fn into_status(self) -> Status
	{
		self.status
	}

	/// Create a new [`Delete`] response.
	#[allow(dead_code)]
	pub const fn new(status: Status) -> Self
	{
		Self { status }
	}

	/// The [`Status`] of the login request.
	#[allow(dead_code)]
	pub const fn status(&self) -> &Status
	{
		&self.status
	}
}
