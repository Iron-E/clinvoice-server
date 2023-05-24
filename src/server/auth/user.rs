//! Contains the definition for what a [`User`] row in the [`Database`](sqlx::Database) is.

#![allow(clippy::std_instead_of_core)]

mod auth_user;

use sqlx::FromRow;
use winvoice_schema::Id;

/// Corresponds to the `users` table in the [`winvoice-server`](crate) database.
#[derive(Clone, Debug, Default, Eq, FromRow, Hash, Ord, PartialEq, PartialOrd)]
pub struct User
{
	/// The [`User`]'s [`Employee`](winvoice_schema::Employee) [`Id`], if they are employed.
	employee_id: Option<Id>,

	/// The [`Id`] of the [`User`].
	id: Id,

	/// The role of the [`User`]. Controls permissions.
	role: String,

	/// Get the [`User`]'s [`argon2`]-hashed password.
	password: String,

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
		username: String,
	) -> Self
	{
		Self { employee_id, id, role, password, username }
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

	/// Get the [`User`]'s username.
	pub fn username(&self) -> &str
	{
		self.username.as_ref()
	}
}
