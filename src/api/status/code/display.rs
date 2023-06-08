use core::fmt::{Display, Formatter, Result};

use super::Code;

impl Display for Code
{
	fn fmt(&self, f: &mut Formatter<'_>) -> Result
	{
		match self
		{
			Self::ApiVersionMismatch =>
			{
				"The API version specified in the version header is incompatible with the version \
				 running on the server"
			},
			Self::BadArguments =>
			{
				"Specific arguments that were used to start the server were not accepted by the \
				 database. If you are a user, please contact an administrator"
			},
			Self::CryptError => "An error occurred while decrypting sensitive data",
			Self::Database => "There was an issue while interfacing with the database adapter",
			Self::InvalidCredentials =>
			{
				"There was an attempt to log in, but it failed because the credentials provided \
				 were incorrect"
			},
			Self::EncodingError => "An error occurred while attempting to de/encode a value",
			Self::LoginError =>
			{
				"Valid credentials were provided, and then an error occurred when attempting to \
				 login"
			},
			Self::Other => "An unknown operation occurred",
			Self::PermissionsError =>
			{
				"An error occurred while attempting to resolve the permissions of this request's \
				 active user"
			},
			Self::SqlError =>
			{
				"The SQL which was generated from a `winvoice_match` was incorrect. This is likely \
				 a bug in Winvoice"
			},
			Self::Success => "The requested operation has completed without error.",
			Self::Unauthorized =>
			{
				"A user has attempted to perform an operation while not having the correct \
				 permissions"
			},
		}
		.fmt(f)
	}
}
