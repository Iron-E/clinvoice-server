//! Contains [`From`] implementations for a [`Login`].

use super::Login;
use crate::api::{Code, Status};

impl From<Code> for Login
{
	fn from(code: Code) -> Self
	{
		Self::new(Status::from(code), None)
	}
}

impl From<Status> for Login
{
	fn from(status: Status) -> Self
	{
		Self::new(status, None)
	}
}
