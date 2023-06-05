//! Implementations for [`AsRef`] for [`Logout`]

use super::Logout;
use crate::api::Code;

impl AsRef<Code> for Logout
{
	fn as_ref(&self) -> &Code
	{
		self.status.as_ref()
	}
}
