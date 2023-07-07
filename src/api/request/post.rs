//! Contains a request to [retrieve](winvoice_adapter::Retrievable)

use serde::{Deserialize, Serialize};

/// The request to create some information.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Post<Args>
{
	/// See [`Retrieve::condition`]
	args: Args,
}

impl<Args> Post<Args>
{
	/// Create a new GET request body.
	#[allow(dead_code)]
	pub const fn new(args: Args) -> Self
	{
		Self { args }
	}

	/// The condition used to filter which entities should be retrieved.
	///
	/// # See also
	///
	/// * [`winvoice_match`]
	/// * [`winvoice_server::api::match`](crate::match)
	#[allow(dead_code)]
	pub const fn args(&self) -> &Args
	{
		&self.args
	}

	/// HACK: can't be an `Into` impl because rust-lang/rust#31844
	///
	/// # See also
	///
	/// * [`Retrieve::condition`]
	#[allow(clippy::missing_const_for_fn)] // destructor cannot be evaluated at compile-time
	pub fn into_args(self) -> Args
	{
		self.args
	}
}
