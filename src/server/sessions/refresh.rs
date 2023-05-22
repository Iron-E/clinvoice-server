//! Contains data regarding what is stored when a user logs in.

use aes_gcm::{aead::Result, Aes256Gcm, Key};
use time::{Duration, OffsetDateTime};

use crate::encrypted::Encrypted;

/// Represents a user who has successfully logged in, and may *stay* logged in.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Refresh
{
	/// The [`DateTime`] that this session will start to be rejected by the server.
	expires: OffsetDateTime,

	/// The [`Password`] for the session.
	password: Encrypted,

	/// The username of the user who has logged in.
	username: Encrypted,
}

impl Refresh
{
	/// Create new [`Refresh`] data. The `username` and `password` are assumed to be *plaintext*,
	/// so that they can be [`Encrypted`] by this method using the `key`.
	///
	/// The `ttl` ensures that the [`Refresh`] data is only valid for a bounded [`Duration`], after
	/// which it will be discarded by the server.
	pub fn new<TUsername, TPassword>(
		username: TUsername,
		password: TPassword,
		key: &Key<Aes256Gcm>,
		ttl: Duration,
	) -> Result<Self>
	where
		TPassword: AsRef<[u8]>,
		TUsername: AsRef<[u8]>,
	{
		let aes_username = Encrypted::new(username, key)?;
		let aes_password = Encrypted::new(password, key)?;
		Ok(Self {
			expires: OffsetDateTime::now_utc() + ttl,
			password: aes_password,
			username: aes_username,
		})
	}

	/// Get the expiry [`OffsetDateTime`] for this [`Refresh`]
	pub const fn expires(&self) -> OffsetDateTime
	{
		self.expires
	}

	/// Get the [`Encrypted`] password for this [`Refresh`]
	pub const fn password(&self) -> &Encrypted
	{
		&self.password
	}

	/// Get the [`Encrypted`] username for this [`Refresh`]
	pub const fn username(&self) -> &Encrypted
	{
		&self.username
	}
}
