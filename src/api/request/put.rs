//! Contains a request to [retrieve](winvoice_adapter::Retrievable)

use serde::{Deserialize, Serialize};

/// The request to create some information.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Put<Args>
{
	/// The arguments for the entity which will be created.
	args: Args,
}

impl<Args> Put<Args>
{
	/// Create a new PUT request body.
	#[allow(dead_code)]
	pub const fn new(args: Args) -> Self
	{
		Self { args }
	}

	/// The arguments for the entity which will be created.
	#[allow(dead_code)]
	pub const fn args(&self) -> &Args
	{
		&self.args
	}

	/// The arguments for the entity which will be created.
	#[allow(clippy::missing_const_for_fn)] // destructor cannot be evaluated at compile-time
	pub fn into_args(self) -> Args
	{
		self.args
	}
}
