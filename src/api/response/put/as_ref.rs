//! Implementations for [`AsRef`] for [`Retrieve`]

use super::Put;
use crate::api::Code;

impl<T> AsRef<Code> for Put<T>
{
	fn as_ref(&self) -> &Code
	{
		self.status.as_ref()
	}
}
