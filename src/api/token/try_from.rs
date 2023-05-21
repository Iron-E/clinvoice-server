//! Allows converting attempting to convert an arbitrary [slice](std::slice) of [bytes](u8) into a
//! [`Token`].

use core::fmt;

use super::{Token, Uuid, MIDDLE};

/// The [`Error`](core::error::Error) for parsing a [`Token`] from bytes.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Error
{
	Aes(aes_gcm::aead::Error),
	Uuid(uuid::Error),
}

impl From<aes_gcm::aead::Error> for Error
{
	fn from(value: aes_gcm::aead::Error) -> Self
	{
		Self::Aes(value)
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
			Self::Aes(e) => e.fmt(f),
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
		let key: [u8; 32] = bytes[MIDDLE..].try_into().map_err(|_| aes_gcm::aead::Error)?;
		Ok(Self::new(uuid, &key.into()))
	}
}
