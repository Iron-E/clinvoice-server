//! Contains the definition for what a [`User`] row in the [`Database`](sqlx::Database) is.

#![cfg_attr(feature = "bin", allow(clippy::std_instead_of_core))]

mod auth_user;

use serde::{Deserialize, Serialize, Serializer};
use winvoice_schema::{
	chrono::{DateTime, Utc},
	Id,
};

/// Corresponds to the `users` table in the [`winvoice_server`] database.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[cfg_attr(feature = "bin", derive(sqlx::FromRow))]
pub struct User
{
	/// The [`User`]'s [`Employee`](winvoice_schema::Employee) [`Id`], if they are employed.
	employee_id: Option<Id>,

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

	/// The [`Id`] of the [`Role`](super::Role) assigned to the [`User`].
	role_id: Id,

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
	pub const fn new(
		employee_id: Option<Id>,
		id: Id,
		password: String,
		password_expires: Option<DateTime<Utc>>,
		role_id: Id,
		username: String,
	) -> Self
	{
		Self { employee_id, id, role_id, password, password_expires, username }
	}

	/// The [`User`]'s [`Employee`](winvoice_schema::Employee) [`Id`], if they are employed.
	pub const fn employee_id(&self) -> Option<i64>
	{
		self.employee_id
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
	pub const fn role_id(&self) -> Id
	{
		self.role_id
	}

	/// Get the [`User`]'s username.
	pub fn username(&self) -> &str
	{
		self.username.as_ref()
	}
}
