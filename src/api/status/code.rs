//! This module contains data used in reporting the specific code which resulted from an action.

mod display;
mod from;
mod try_from;

use serde::{Deserialize, Serialize};

/// The specific outcome of an operation.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(into = "u8", try_from = "u8")]
pub enum Code
{
	/// The API version header was not sent with a particular request.
	ApiVersionHeaderMissing = 12,

	/// The API version specified in the version header is incompatible with the version running on
	/// the server.
	ApiVersionMismatch = 10,

	/// Specific arguments that were used to start the server were not accepted by the
	/// database.
	BadArguments = 4,

	/// An error occurred while decrypting sensitive data.
	CryptError = 5,

	/// There was an issue while interfacing with [`sqlx`].
	Database = 7,

	/// An error occurred while attempting to de/encode a value.
	EncodingError = 6,

	/// An error occurred while attempting to exchange [currencies](winvoice_schema::Currency).
	ExchangeError = 15,

	/// There was an attempt to log in, but it failed because the credentials provided were not
	/// accepted by the database.
	InvalidCredentials = 2,

	/// The requested operation has completed without error, but some actions had to be skipped
	/// because of missing permissions.
	SuccessForPermissions = 14,

	/// The requested operation has completed without error.
	Success = 1,

	/// Valid credentials were provided, and then an error occurred when attempting to login.
	LoginError = 9,

	/// An uncategorized type of action was taken.
	#[default]
	Other = 0,

	/// A valid login was presented, but the password has expired and must be changed.
	PasswordExpired = 13,

	/// An error occurred while attempting to resolve the permissions of this request's active
	/// user.
	PermissionsError = 11,

	/// The SQL which was generated from a [`winvoice_match`] was incorrect. This is likely a bug
	/// in Winvoice.
	SqlError = 8,

	/// A user has attempted to perform an operation on the database while not having the correct
	/// permissions.
	Unauthorized = 3,
}
