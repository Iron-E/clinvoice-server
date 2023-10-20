//! Contains [`From`] implementations for a [`LoginResponse`].

use sqlx::Error as SqlxError;
use winvoice_schema::chrono::OutOfRangeError;

use super::LoginResponse;
use crate::api::{Code, Status};

impl From<Code> for LoginResponse
{
	fn from(code: Code) -> Self
	{
		Self::from(Status::from(code))
	}
}

impl From<argon2::password_hash::Error> for LoginResponse
{
	fn from(error: argon2::password_hash::Error) -> Self
	{
		Self::from(Status::from(&error))
	}
}

impl From<OutOfRangeError> for LoginResponse
{
	fn from(error: OutOfRangeError) -> Self
	{
		Self::from(Status::from(&error))
	}
}

impl From<SqlxError> for LoginResponse
{
	fn from(error: SqlxError) -> Self
	{
		Self::from(Status::from(&error))
	}
}

impl From<Status> for LoginResponse
{
	fn from(status: Status) -> Self
	{
		Self::new(status.code().into(), status, None)
	}
}
