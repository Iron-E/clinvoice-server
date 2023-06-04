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
			                       administrator"
				.fmt(f),
			Self::CryptError => "An error occurred while decrypting sensitive data".fmt(f),
			Self::Database =>
			{
				"There was an issue while interfacing with the database adapter".fmt(f)
			},
			Self::InvalidCredentials => "There was an attempt to log in, but it failed because \
			                             the credentials provided were incorrect"
				.fmt(f),
			Self::EncodingError => "An error occurred while attempting to de/encode a value".fmt(f),
			Self::LoggedIn => "A user has been logged in".fmt(f),
			Self::LoggedOut => "A user has been logged out".fmt(f),
			Self::LoginError => "Valid credentials were provided, and then an error occurred when \
			                     attempting to login"
				.fmt(f),
			Self::Other => "An unknown operation occurred".fmt(f),
			Self::SqlError => "The SQL which was generated from a `winvoice_match` was incorrect. \
			                   This is likely a bug in Winvoice"
				.fmt(f),
			Self::Unauthorized => "A user has attempted to perform an operation while not having \
			                       the correct permissions"
				.fmt(f),
		}
	}
}
