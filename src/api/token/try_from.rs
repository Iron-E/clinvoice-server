//! Allows converting attempting to convert an arbitrary [slice](std::slice) of [bytes](u8) into a
//! [`Token`].

use core::{array::TryFromSliceError, fmt};

use super::{Token, Uuid, MIDDLE};

/// The [`Error`](core::error::Error) for parsing a [`Token`] from bytes.
#[derive(Clone, Debug)]
pub enum Error
{
	Array(TryFromSliceError),
	Uuid(uuid::Error),
}

impl From<TryFromSliceError> for Error
{
	fn from(value: TryFromSliceError) -> Self
	{
		Self::Array(value)
	}
}

impl From<uuid::Error> for Error
{
	fn from(value: uuid::Error) -> Self
	{
		Self::Uuid(value)
	}
}

impl fmt::Display for Error
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
	{
		match self
		{
			Self::Array(e) => e.fmt(f),
			Self::Uuid(e) => e.fmt(f),
		}
	}
}

impl std::error::Error for Error {}

impl TryFrom<&[u8]> for Token
{
	type Error = Error;

	fn try_from(bytes: &[u8]) -> Result<Self, Self::Error>
	{
		let uuid = Uuid::from_slice(&bytes[..MIDDLE])?;
		let key: [u8; 32] = bytes[MIDDLE..].try_into()?;
		Ok(Self::new(uuid, &key.into()))
	}
}
