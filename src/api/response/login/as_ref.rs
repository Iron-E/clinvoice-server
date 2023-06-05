//! Implementations for [`AsRef`] for [`Login`]

use super::Login;
use crate::api::Code;

impl AsRef<Code> for Login
{
	fn as_ref(&self) -> &Code
	{
		self.status.as_ref()
	}
}
