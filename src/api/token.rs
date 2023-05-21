//! This module contains data types which are used by [`winvoice-server`](crate) to refer to a
//! unique user identity.

mod try_from;

use aes_gcm::{Aes256Gcm, Key};
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use uuid::Uuid;

/// To the left of this index is the `uuid`, to the right is the `key`.
const MIDDLE: usize = 16;

/// Data which is used to refer to a unique user identity.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Token(#[serde(with = "BigArray")] [u8; 48]);

impl Token
{
	pub fn new(uuid: Uuid, key: &Key<Aes256Gcm>) -> Self
	{
		let mut token: [u8; 48] = [0; 48];
		let (token_uuid, token_key) = token.split_at_mut(MIDDLE);
		token_uuid.copy_from_slice(uuid.as_bytes());
		token_key.copy_from_slice(key);
		Self(token)
	}

	/// Get the key part of the [`Token`].
	pub fn key(&self) -> Key<Aes256Gcm>
	{
		let array: [u8; 32] = self.0[MIDDLE..].try_into().unwrap();
		array.into()
	}

	/// Get the [`Uuid`] part of the [`Token`]
	pub fn uuid(&self) -> Uuid
	{
		Uuid::from_bytes(self.0[..MIDDLE].try_into().unwrap())
	}
}

#[cfg(test)]
mod tests
{
	use aes_gcm::{Aes256Gcm, Key};
	use pretty_assertions::assert_eq;

	use super::{Token, Uuid};

	#[test]
	fn new()
	{
		let key: Key<Aes256Gcm> = [1; 32].into();
		let uuid = Uuid::new_v4();

		let token = Token::new(uuid, &key);
		assert_eq!(token.key(), key);
		assert_eq!(token.uuid(), uuid);
	}
}
