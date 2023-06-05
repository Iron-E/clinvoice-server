//! Contains [`From`] implementations for a [`LoginResponse`].

use super::LogoutResponse;
use crate::api::Code;

impl From<Code> for LogoutResponse
{
	fn from(code: Code) -> Self
	{
		Self::new(code.into(), code.into())
	}
}
