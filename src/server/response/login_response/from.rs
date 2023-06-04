//! Contains [`From`] implementations for a [`LoginResponse`].

use sqlx::Error as SqlxError;

use super::LoginResponse;
use crate::api::{Code, Status};

impl From<Code> for LoginResponse
{
	fn from(code: Code) -> Self
	{
		Self::new(code.into(), code.into())
	}
}

impl From<argon2::password_hash::Error> for LoginResponse
{
	fn from(error: argon2::password_hash::Error) -> Self
	{
		let status = Status::from(&error);
		Self::new(status.code().into(), status)
	}
}

impl From<SqlxError> for LoginResponse
{
	fn from(error: SqlxError) -> Self
	{
		let status = Status::from(&error);
		Self::new(status.code().into(), status)
	}
}
