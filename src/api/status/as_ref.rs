//! Implementations for [`AsRef`] for [`Status`]

use super::{Code, Status};

impl AsRef<Code> for Status
{
	fn as_ref(&self) -> &Code
	{
		&self.code
	}
}
