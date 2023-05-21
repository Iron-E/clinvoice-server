//! This module contains data used in reporting the specific code which resulted from an action.

mod display;

use serde::{Deserialize, Serialize};

/// The specific outcome of an operation.
#[derive(
	Clone, Copy, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize,
)]
pub enum Code
{
	/// Specific arguments that were used to start the server were not accepted by the
	/// database.
	BadArguments = 5,

	/// An error occurred while decrypting sensitive data.
	DecryptError = 9,

	/// There was an attempt to log in, but it failed because the credentials provided were not
	/// accepted by the database.
	InvalidCredentials = 2,

	/// An error occurred while encrypting sensitive data.
	EncryptError = 8,

	/// A user has successfully logged in.
	LoggedIn = 1,

	/// A user has successfully logged out.
	LoggedOut = 4,

	/// A [`Uuid`](uuid::Uuid) was sent with a request that was not formatted correctly.
	MalformedUuid = 6,

	/// An uncategorized type of action was taken.
	#[default]
	Other = 0,

	/// A valid [`Uuid`](uuid::Uuid) was sent for authentication but did not exist on the
	/// server.
	SessionNotFound = 7,

	/// A user has attempted to perform an operation on the database while not having the correct
	/// permissions.
	Unauthorized = 3,
}
