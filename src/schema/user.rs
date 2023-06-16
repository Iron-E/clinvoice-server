//! Contains the definition for what a [`User`] row in the [`Database`](sqlx::Database) is.

#![cfg_attr(feature = "bin", allow(clippy::std_instead_of_core))]

#[cfg(feature = "bin")]
mod auth_user;
#[cfg(feature = "postgres")]
mod date_time_ext;
#[cfg(feature = "bin")]
mod from_row;

use std::sync::OnceLock;

use argon2::{
	password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
	Argon2,
};
use serde::{Deserialize, Serialize, Serializer};
use winvoice_schema::{
	chrono::{DateTime, Duration, Utc},
	Employee,
	Id,
};

use super::Role;
use crate::dyn_result::DynResult;

static ARGON: OnceLock<Argon2> = OnceLock::new();

/// Corresponds to the `users` table.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct User
{
	/// The [`User`]'s [`Employee`](winvoice_schema::Employee) [`Id`], if they are employed.
	employee: Option<Employee>,

	/// The [`Id`] of the [`User`].
	id: Id,

	/// The [hashed](argon2) password.
	///
	/// # `POST`/`PATCH`
	///
	/// The password in plaintext, which *will* be [hashed](argon2) and stored in the
	/// [`Database`](sqlx::Database) by [`winvoice_server`].
	#[serde(serialize_with = "serialize_password")]
	password: String,

	/// The [`DateTime`] that the `password` was set. Used to enforce password rotation.
	password_expires: Option<DateTime<Utc>>,

	/// The [`Role`] assigned to the [`User`].
	role: Role,

	/// Get the [`User`]'s username.
	username: String,
}

/// A custom serializer for the [`User`] password which prevents anyone from ever seeing the
/// password [hash](argon2), and instead prompts them with the intended use of the field when it is
/// visible.
fn serialize_password<S>(_: &str, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	serializer.serialize_str("[replace this text to set new password]")
}

impl User
{
	/// Create a new [`User`].
	pub fn new(
		employee: Option<Employee>,
		id: Id,
		password: String,
		role: Role,
		username: String,
	) -> DynResult<Self>
	{
		let password_expires = role
			.password_ttl()
			.map(|ttl| Duration::from_std(ttl).map(|d| Utc::now() + d))
			.transpose()?;

		let argon = ARGON.get_or_init(Argon2::default);
		let salt = SaltString::generate(&mut OsRng);
		argon.hash_password(password.as_bytes(), &salt).map_or_else(
			|e| Err(format!("{e}").into()),
			|hash| {
				Ok(Self {
					employee,
					id,
					role,
					password: hash.to_string(),
					password_expires,
					username,
				})
			},
		)
	}

	/// The [`User`]'s [`Employee`](winvoice_schema::Employee) [`Id`], if they are employed.
	pub const fn employee(&self) -> Option<&Employee>
	{
		self.employee.as_ref()
	}

	/// The [`Id`] of the [`User`].
	pub const fn id(&self) -> Id
	{
		self.id
	}

	/// Get the [`User`]'s [`argon2`]-hashed password.
	pub fn password(&self) -> &str
	{
		self.password.as_ref()
	}

	/// Get the [`DateTime`] that the `password` was set. Used to enforce password rotation.
	pub const fn password_expires(&self) -> Option<DateTime<Utc>>
	{
		self.password_expires
	}

	/// The [`Id`] of the [`Role`](super::Role) assigned to the [`User`].
	pub const fn role(&self) -> &Role
	{
		&self.role
	}

	/// Get the [`User`]'s username.
	pub fn username(&self) -> &str
	{
		self.username.as_ref()
	}
}
