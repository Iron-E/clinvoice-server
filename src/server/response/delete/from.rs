//! Contains [`From`] implementations for a [`DeleteResponse`].

use argon2::password_hash::Error as HashError;
use sqlx::Error as SqlxError;

use super::DeleteResponse;
use crate::api::{Code, Status};

impl From<Code> for DeleteResponse
{
	fn from(code: Code) -> Self
	{
		Self::from(Status::from(code))
	}
}

impl From<HashError> for DeleteResponse
{
	fn from(error: HashError) -> Self
	{
		Self::from(Status::from(&error))
	}
}

impl From<SqlxError> for DeleteResponse
{
	fn from(error: SqlxError) -> Self
	{
		Self::from(Status::from(&error))
	}
}

impl From<Status> for DeleteResponse
{
	fn from(status: Status) -> Self
	{
		Self::new(status.code().into(), status)
	}
}
