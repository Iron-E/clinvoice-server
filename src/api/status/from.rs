//! Contains the [`From`] implementations for [`Status`].

use sqlx::Error as SqlxError;

use super::{Code, Status};

impl From<Code> for Status
{
	fn from(code: Code) -> Self
	{
		Self { code, message: code.to_string() }
	}
}

impl From<&argon2::password_hash::Error> for Status
{
	fn from(error: &argon2::password_hash::Error) -> Self
	{
		Self::new(error.into(), error.to_string())
	}
}

impl From<&SqlxError> for Status
{
	fn from(error: &SqlxError) -> Self
	{
		Self::new(error.into(), error.to_string())
	}
}
