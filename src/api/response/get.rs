//! This module contains the response for a [retrieve](winvoice_adapter::Retrievable) operation.

mod as_ref;
mod from;

use serde::{Deserialize, Serialize};

use crate::api::Status;

/// The response for a GET / retrieve operation.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Get<T>
{
	/// The entities in the database which [match](winvoice_match)ed the
	/// [request](crate::api::request::Retrieve) parameters.
	entities: Vec<T>,

	/// The [`Status`] of this request.
	status: Status,
}

impl<T> Get<T>
{
	/// Create a new [`Retrieve`] response.
	pub const fn new(entities: Vec<T>, status: Status) -> Self
	{
		Self { entities, status }
	}

	/// The entities in the database which [match](winvoice_match)ed the
	/// [request](crate::api::request::Retrieve) parameters.
	#[allow(dead_code)]
	pub fn entities(&self) -> &[T]
	{
		self.entities.as_ref()
	}

	/// The [`Status`] of the logout request.
	#[allow(dead_code)]
	pub const fn status(&self) -> &Status
	{
		&self.status
	}
}
