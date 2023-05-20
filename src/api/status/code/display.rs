use core::fmt::{Display, Formatter, Result};

use super::Code;

impl Display for Code
{
	fn fmt(&self, f: &mut Formatter<'_>) -> Result
	{
		match self
		{
			Self::InvalidCredentials => "There was an attempt to log in, but it failed because \
			                             the credentials provided were incorrect"
				.fmt(f),
			Self::LoggedIn => "A user has been logged in".fmt(f),
			Self::LoggedOut => "A user has been logged out".fmt(f),
			Self::Other => "An unknown operation occurred".fmt(f),
			Self::Unauthorized => "A user has attempted to perform an operation while not having \
			                       the correct permissions"
				.fmt(f),
		}
	}
}
