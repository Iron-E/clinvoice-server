//! Contains the definition for what a [`User`] row in the [`Database`](sqlx::Database) is.

#![cfg_attr(feature = "bin", allow(clippy::std_instead_of_core))]

mod auth_user;

use serde::{Deserialize, Serialize};
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

	/// Get the [`User`]'s [`argon2`]-hashed password.
	password: String,

	/// The [`DateTime`] that the `password` was set. Used to enforce password rotation.
	password_expires: Option<DateTime<Utc>>,

	/// The role of the [`User`]. Controls permissions.
	role: String,

	/// Get the [`User`]'s username.
	username: String,
}

impl User
{
	/// Create a new [`User`].
	pub const fn new(
		employee_id: Option<Id>,
		id: Id,
		role: String,
		password: String,
		password_expires: Option<DateTime<Utc>>,
		username: String,
	) -> Self
	{
		Self { employee_id, id, role, password, password_expires, username }
	}

	/// The [`User`]'s [`Employee`](winvoice_schema::Employee) [`Id`], if they are employed.
	pub const fn employee_id(&self) -> Option<i64>
	{
		self.employee_id
	}

	/// The [`Id`] of the [`User`].
	pub const fn id(&self) -> i64
	{
		self.id
	}

	/// The role of the [`User`]. Controls permissions.
	pub fn role(&self) -> &str
	{
		self.role.as_ref()
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

	/// Get the [`User`]'s username.
	pub fn username(&self) -> &str
	{
		self.username.as_ref()
	}
}

/// A [`User`] which has specialized implementations for the [`Postgres`](sqlx::Postgres) database.
#[cfg(feature = "postgres")]
pub struct PgUser;
