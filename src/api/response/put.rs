//! This module contains the response for a [retrieve](winvoice_adapter::Retrievable) operation.

mod as_ref;
mod from;

use serde::{Deserialize, Serialize};

use crate::api::Status;

/// The response for a PUT / create operation.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Put<T>
{
	/// The entity in the database which was created as a result of the operation.
	entity: Option<T>,

	/// The [`Status`] of this request.
	status: Status,
}

impl<T> Put<T>
{
	/// Create a new [`Retrieve`] response.
	pub const fn new(entity: Option<T>, status: Status) -> Self
	{
		Self { entity, status }
	}

	/// The entities in the database which [match](winvoice_match)ed the
	/// [request](crate::api::request::Retrieve) parameters.
	#[allow(dead_code)]
	pub const fn entity(&self) -> Option<&T>
	{
		self.entity.as_ref()
	}

	/// The entities in the database which [match](winvoice_match)ed the
	/// [request](crate::api::request::Retrieve) parameters.
	#[allow(dead_code)]
	pub fn into_entity(self) -> Option<T>
	{
		self.entity
	}

	/// The entities in the database which [match](winvoice_match)ed the
	/// [request](crate::api::request::Retrieve) parameters.
	#[allow(dead_code)]
	pub fn into_status(self) -> Status
	{
		self.status
	}

	/// The [`Status`] of the logout request.
	#[allow(dead_code)]
	pub const fn status(&self) -> &Status
	{
		&self.status
	}
}
