//! This module contains the response for a login.

mod as_ref;
mod from;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::api::Status;

/// The DELETE & [`winvoice_adapter::Deletable::delete`] request response.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Export
{
	/// The exported [`Job`](winvoice_schema::Job)s.
	exported: HashMap<String, String>,

	/// The [`Status`] of the login request.
	status: Status,
}

impl Export
{
	/// The [`Status`] of the login request.
	#[allow(dead_code)]
	pub const fn exported(&self) -> &HashMap<String, String>
	{
		&self.exported
	}

	/// The [`Status`] of the login request.
	#[allow(dead_code)]
	pub fn into_status(self) -> Status
	{
		self.status
	}

	/// Create a new [`Export`] response.
	#[allow(dead_code)]
	pub const fn new(exported: HashMap<String, String>, status: Status) -> Self
	{
		Self { exported, status }
	}

	/// The [`Status`] of the login request.
	#[allow(dead_code)]
	pub const fn status(&self) -> &Status
	{
		&self.status
	}
}
