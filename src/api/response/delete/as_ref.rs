//! Implementations for [`AsRef`] for [`Retrieve`]

use super::Delete;
use crate::api::Code;

impl AsRef<Code> for Delete
{
	fn as_ref(&self) -> &Code
	{
		self.status.as_ref()
	}
}
