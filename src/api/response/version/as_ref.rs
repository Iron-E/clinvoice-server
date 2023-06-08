//! Implementations for [`AsRef`] for [`Version`]

use super::Version;
use crate::api::Code;

impl AsRef<Code> for Version
{
	fn as_ref(&self) -> &Code
	{
		self.status.as_ref()
	}
}
