use core::fmt::{Display, Formatter, Result};

use super::Code;

impl Display for Code
{
	fn fmt(&self, f: &mut Formatter<'_>) -> Result
	{
		match self
		{
			Self::BadArguments => "Specific arguments that were used to start the server were not \
			                       accepted by the database. If you are a user, please contact an \
			                       administrator."
				.fmt(f),
			Self::DbAdapterError => "There mau have been an error in an `sqlx` adapter, or a \
			                         `sqlx` Connection` became corrupted."
				.fmt(f),
			Self::DbConnectionSevered =>
			{
				"The connection to the database was unexpectedly cut short".fmt(f)
			},
			Self::DbConnectTimeout => "The server was unable to establish a connection with the \
			                           database because the task timed out"
				.fmt(f),
			Self::DbIoError => "An IO error occured while communicating with the database".fmt(f),
			Self::DbTlsError =>
			{
				"An error involving TLS occurred while communicating with the database".fmt(f)
			},
			Self::DecodeError => "An error occurred while trying to decode a value".fmt(f),
			Self::DecryptError => "An error occurred while decrypting sensitive data.".fmt(f),
			Self::InvalidCredentials => "There was an attempt to log in, but it failed because \
			                             the credentials provided were incorrect"
				.fmt(f),
			Self::EncryptError => "An error occurred while encrypting sensitive data.".fmt(f),
			Self::LoggedIn => "A user has been logged in".fmt(f),
			Self::LoggedOut => "A user has been logged out".fmt(f),
			Self::Other => "An unknown operation occurred".fmt(f),
			Self::SessionNotFound =>
			{
				"A valid uuid was sent for authentication but did not exist on the server".fmt(f)
			},
			Self::SqlError => "The SQL which was generated from a `winvoice_match` was incorrect. \
			                   This is likely a bug in Winvoice"
				.fmt(f),
			Self::Unauthorized => "A user has attempted to perform an operation while not having \
			                       the correct permissions"
				.fmt(f),
		}
	}
}
