//! Implementations for [`AsRef`] for [`Retrieve`]

use super::Get;
use crate::api::Code;

impl<T> AsRef<Code> for Get<T>
{
	fn as_ref(&self) -> &Code
	{
		self.status.as_ref()
	}
}
