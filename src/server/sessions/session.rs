//! Contains data regarding what is stored when a user logs in.

mod hash;
mod partial_eq;
mod partial_ord;

use aes_gcm::{aead::Result, Aes256Gcm, Key};
use sqlx::{Database, Pool};
use winvoice_schema::chrono::{DateTime, Utc};

use crate::encrypted::Encrypted;

/// Represents a user who has successfully logged in, and may *stay* logged in.
#[derive(Clone, Debug, Default)]
pub struct Session<Db>
where
	Db: Database,
{
	/// This's [`Session`]'s connection to the [`Database`].
	connection: Option<Pool<Db>>,

	/// The [`DateTime`] that this session was created. Stored for the purposes of ensuring expiry
	/// is done on time.
	date: DateTime<Utc>,

	/// The [`Password`] for the session.
	password: Encrypted,

	/// The username of the user who has logged in.
	username: Encrypted,
}

impl<Db> Session<Db>
where
	Db: Database,
{
	pub(super) fn new<TUsername, TPassword>(
		username: TUsername,
		password: TPassword,
		key: &Key<Aes256Gcm>,
		connection: Pool<Db>,
	) -> Result<Self>
	where
		TPassword: AsRef<[u8]>,
		TUsername: AsRef<[u8]>,
	{
		let aes_username = Encrypted::new(username, key)?;
		let aes_password = Encrypted::new(password, key)?;
		Ok(Self {
			connection: connection.into(),
			date: Utc::now(),
			password: aes_password,
			username: aes_username,
		})
	}
}
