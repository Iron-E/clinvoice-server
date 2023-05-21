//! This module contains data types which are used by [`winvoice-server`](crate) to refer to a
//! unique user identity.

use core::fmt;
use std::string::FromUtf8Error;

use aes_gcm::{
	aead::{Aead, Error as AeadError, OsRng},
	AeadCore,
	Aes256Gcm,
	Key,
	KeyInit,
	Nonce,
};
use serde::{Deserialize, Serialize};

/// To the left of this index is the `nonce`, to the right is the `password`.
const MIDDLE: usize = 12;

/// Data which is used to refer to a unique user identity.
#[derive(Clone, Default, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Encrypted(Vec<u8>);

/// The [`Error`](core::error::Error) for decrypting [`Encrypted`] content.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error
{
	Aes(AeadError),
	Utf8(FromUtf8Error),
}

impl From<AeadError> for Error
{
	fn from(value: AeadError) -> Self
	{
		Self::Aes(value)
	}
}

impl From<FromUtf8Error> for Error
{
	fn from(value: FromUtf8Error) -> Self
	{
		Self::Utf8(value)
	}
}

impl fmt::Display for Error
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
	{
		match self
		{
			Self::Aes(e) => e.fmt(f),
			Self::Utf8(e) => e.fmt(f),
		}
	}
}

impl std::error::Error for Error {}

impl Encrypted
{
	/// Encrypts the `content` with the `key` using [`Aes256Gcm`].
	pub fn new<T>(content: T, key: &Key<Aes256Gcm>) -> Result<Self, AeadError>
	where
		T: AsRef<[u8]>,
	{
		let cipher = Aes256Gcm::new(key);
		let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

		let mut encrypted = cipher.encrypt(&nonce, content.as_ref())?;
		let index = encrypted.len();
		encrypted.extend_from_slice(&nonce);
		encrypted.rotate_left(index);

		Ok(Self(encrypted))
	}

	/// Get the content that was [`Encrypted`].
	pub fn decrypt(&self, key: &Key<Aes256Gcm>) -> Result<String, Error>
	{
		let cipher = Aes256Gcm::new(key);
		let nonce: Nonce<_> = self.nonce().into();

		println!("{:?}", self.0);
		let bytes = cipher.decrypt(&nonce, self.0[MIDDLE..].as_ref())?;
		let decrypted = String::from_utf8(bytes)?;
		Ok(decrypted)
	}

	/// Get the nonce of the [`Encrypted`] content.
	pub fn nonce(&self) -> [u8; 12]
	{
		self.0[..MIDDLE].try_into().unwrap()
	}
}

#[cfg(test)]
mod tests
{
	use aes_gcm::{aead::OsRng, Aes256Gcm, KeyInit};

	use super::Encrypted;

	#[test]
	fn new()
	{
		let key = Aes256Gcm::generate_key(OsRng);
		let plaintext = "my password".to_owned();

		let content = Encrypted::new(&plaintext, &key).unwrap();
		assert_eq!(content.decrypt(&key), Ok(plaintext));
	}
}
