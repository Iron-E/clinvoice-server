//! This module holds data for the columns of the [`User`](crate::schema::User) table.

mod columns_to_sql;
mod table_to_sql;

use serde::{Deserialize, Serialize};
use winvoice_adapter::fmt::{TableToSql, WithIdentifier};

/// The names of the columns of the `users` table.
#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct UserColumns<T = &'static str>
{
	/// The name of the `employee_id` column of the `users` table.
	pub employee_id: T,

	/// The name of the `id` column of the `users` table.
	pub id: T,

	/// The name of the `password` column of the `users` table.
	pub password: T,

	/// The name of the `password_expires` column of the `users` table.
	pub password_expires: T,

	/// The name of the `role` column of the `users` table.
	pub role_id: T,

	/// The name of the `username` column of the `users` table.
	pub username: T,
}

impl<T> UserColumns<T>
{
	/// Add a [scope](ExpenseColumns::scope) using the [default alias](TableToSql::default_alias)
	///
	/// # See also
	///
	/// * [`WithIdentifier`].
	#[allow(clippy::missing_const_for_fn)] // destructor cannot be evaluated at compile-time
	pub fn default_scope(self) -> UserColumns<WithIdentifier<char, T>>
	{
		self.scope(UserColumns::DEFAULT_ALIAS)
	}

	/// Returns a [`UserColumns`] which modifies its fields' [`Display`]
	/// implementation to output `{alias}.{column}`.
	///
	/// # See also
	///
	/// * [`WithIdentifier`]
	#[allow(clippy::missing_const_for_fn)] // destructor cannot be evaluated at compile-time
	pub fn scope<Alias>(self, alias: Alias) -> UserColumns<WithIdentifier<Alias, T>>
	where
		Alias: Copy,
	{
		UserColumns {
			employee_id: WithIdentifier(alias, self.employee_id),
			id: WithIdentifier(alias, self.id),
			password: WithIdentifier(alias, self.password),
			password_expires: WithIdentifier(alias, self.password_expires),
			role_id: WithIdentifier(alias, self.role_id),
			username: WithIdentifier(alias, self.username),
		}
	}
}

impl UserColumns<&'static str>
{
	/// The names of the columns in `organizations` without any aliasing.
	pub const fn default() -> Self
	{
		Self {
			employee_id: "employee_id",
			id: "id",
			password: "password",
			password_expires: "password_expires",
			role_id: "role_id",
			username: "username",
		}
	}
}
