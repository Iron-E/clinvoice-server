//! Contains a request to [retrieve](winvoice_adapter::Retrievable)

use serde::{Deserialize, Serialize};

/// The request to [delete](winvoice_adapter::Deletable::delete) some information.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Delete<T>
{
	/// See [`Retrieve::condition`]
	entities: Vec<T>,
}

impl<T> Delete<T>
{
	/// Create a new GET request body.
	#[allow(dead_code)]
	pub const fn new(entities: Vec<T>) -> Self
	{
		Self { entities }
	}

	/// The condition used to filter which entities should be retrieved.
	///
	/// # See also
	///
	/// * [`winvoice_match`]
	/// * [`winvoice_server::api::match`](crate::match)
	#[allow(dead_code)]
	pub fn entities(&self) -> &[T]
	{
		self.entities.as_ref()
	}

	/// HACK: can't be an `Into` impl because rust-lang/rust#31844
	///
	/// # See also
	///
	/// * [`Retrieve::condition`]
	#[allow(clippy::missing_const_for_fn, dead_code)] // destructor cannot be evaluated at compile-time
	pub fn into_entities(self) -> Vec<T>
	{
		self.entities
	}
}
