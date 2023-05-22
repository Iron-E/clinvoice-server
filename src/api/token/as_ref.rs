//! Contains the [`AsRef`] implementation for [`Token`]

use super::Token;

impl AsRef<[u8]> for Token
{
	fn as_ref(&self) -> &[u8]
	{
		self.0.as_ref()
	}
}
