//! This module contains data used in reporting the specific code which resulted from an action.

mod display;
mod from;

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
	CryptError = 6,

	/// There was an issue while interfacing with [`sqlx`].
	Database = 8,

	/// An error occurred while attempting to de/encode a value.
	EncodingError = 7,

	/// There was an attempt to log in, but it failed because the credentials provided were not
	/// accepted by the database.
	InvalidCredentials = 2,

	/// A user has successfully logged in.
	LoggedIn = 1,

	/// A user has successfully logged out.
	LoggedOut = 4,

	/// Valid credentials were provided, and then an error occurred when attempting to login.
	LoginError = 10,

	/// An uncategorized type of action was taken.
	#[default]
	Other = 0,

	/// The SQL which was generated from a [`winvoice_match`] was incorrect. This is likely a bug
	/// in Winvoice.
	SqlError = 9,

	/// A user has attempted to perform an operation on the database while not having the correct
	/// permissions.
	Unauthorized = 3,
}
