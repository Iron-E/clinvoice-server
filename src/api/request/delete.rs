//! Contains a request to [retrieve](winvoice_adapter::Retrievable)

use serde::{Deserialize, Serialize};

/// The request to [delete](winvoice_adapter::Deletable::delete) some information.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Delete<T>
{
	/// The entities which have their deletion requested as a slice.
	entities: Vec<T>,
}

impl<T> Delete<T>
{
	/// Create a new DELETE request body.
	#[allow(dead_code)]
	pub const fn new(entities: Vec<T>) -> Self
	{
		Self { entities }
	}

	/// The entities which have their deletion requested as a slice.
	#[allow(dead_code)]
	pub fn entities(&self) -> &[T]
	{
		self.entities.as_ref()
	}

	/// The entities which have their deletion requested.
	#[allow(clippy::missing_const_for_fn, dead_code)] // destructor cannot be evaluated at compile-time
	pub fn into_entities(self) -> Vec<T>
	{
		self.entities
	}
}
