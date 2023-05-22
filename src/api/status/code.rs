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

	/// There mau have been an error in an [`sqlx`] adapter, or a [`sqlx::Connection`] became
	/// corrupted.
	DbAdapterError = 13,

	/// The connection to the database was unexpectedly cut short.
	DbConnectionSevered = 15,

	/// The server was unable to establish a connection with the [`Database`](sqlx::Database)
	/// because the task timed out.
	DbConnectTimeout = 12,

	/// [`std::io::Error`] while communicating with the [`Database`](sqlx::Database).
	DbIoError = 10,

	/// An error involving TLS occurred while communicating with the [`Database`](sqlx::Database).
	DbTlsError = 14,

	/// An error occurred while attempting to decode a value.
	DecodeError = 11,

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

	/// An uncategorized type of action was taken.
	#[default]
	Other = 0,

	/// A valid [`Uuid`](uuid::Uuid) was sent for authentication but did not exist on the
	/// server.
	SessionNotFound = 7,

	/// The SQL which was generated from a [`winvoice_match`] was incorrect. This is likely a bug
	/// in Winvoice.
	SqlError = 16,

	/// A user has attempted to perform an operation on the database while not having the correct
	/// permissions.
	Unauthorized = 3,
}
