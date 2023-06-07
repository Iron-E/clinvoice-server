//! Contains the [`From`] implementations for [`Status`].

use argon2::password_hash::Error as PasswordError;
use casbin::Error as CasbinError;
use sqlx::Error as SqlxError;

use super::{Code, Status};

impl From<Code> for Status
{
	fn from(code: Code) -> Self
	{
		Self { code, message: code.to_string() }
	}
}

impl From<&CasbinError> for Status
{
	fn from(error: &CasbinError) -> Self
	{
		Self::new(error.into(), error.to_string())
	}
}

impl From<&PasswordError> for Status
{
	fn from(error: &PasswordError) -> Self
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
