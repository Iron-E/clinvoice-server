//! Implementations for [`AsRef`] for [`Retrieve`]

use super::Export;
use crate::api::Code;

impl AsRef<Code> for Export
{
	fn as_ref(&self) -> &Code
	{
		self.status.as_ref()
	}
}
