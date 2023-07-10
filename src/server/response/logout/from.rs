//! Contains [`From`] implementations for a [`LoginResponse`].

use super::LogoutResponse;
use crate::api::{Code, Status};

impl From<Code> for LogoutResponse
{
	fn from(code: Code) -> Self
	{
		Self::from(Status::from(code))
	}
}

impl From<Status> for LogoutResponse
{
	fn from(status: Status) -> Self
	{
		Self::new(status.code().into(), status)
	}
}
