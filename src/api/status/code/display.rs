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
			Self::DatabaseIoError =>
			{
				"An IO error occured while communicating with the database".fmt(f)
			},
			Self::DecryptError => "An error occurred while decrypting sensitive data.".fmt(f),
			Self::InvalidCredentials => "There was an attempt to log in, but it failed because \
			                             the credentials provided were incorrect"
				.fmt(f),
			Self::EncryptError => "An error occurred while encrypting sensitive data.".fmt(f),
			Self::LoggedIn => "A user has been logged in".fmt(f),
			Self::LoggedOut => "A user has been logged out".fmt(f),
			Self::MalformedUuid =>
			{
				"A uuid was sent with a request that was not formatted correctly".fmt(f)
			},
			Self::Other => "An unknown operation occurred".fmt(f),
			Self::SessionNotFound =>
			{
				"A valid uuid was sent for authentication but did not exist on the server".fmt(f)
			},
			Self::Unauthorized => "A user has attempted to perform an operation while not having \
			                       the correct permissions"
				.fmt(f),
		}
	}
}
