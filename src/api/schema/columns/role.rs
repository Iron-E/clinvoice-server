//! This module holds data for the columns of the [`Role`](crate::api::schema::Role) table.

mod columns_to_sql;
mod table_to_sql;

use serde::{Deserialize, Serialize};
use winvoice_adapter::fmt::{TableToSql, WithIdentifier};

/// The names of the columns of the `roles` table.
#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct RoleColumns<T>
{
	/// The name of the `id` column of the `roles` table.
	pub id: T,

	/// The name of the `name` column of the `roles` table.
	pub name: T,

	/// The name of the `password_ttl` column of the `roles` table.
	pub password_ttl: T,
}

impl<T> RoleColumns<T>
{
	/// Add a [scope](ExpenseColumns::scope) using the [default alias](TableToSql::default_alias)
	///
	/// # See also
	///
	/// * [`WithIdentifier`].
	pub fn default_scope(self) -> RoleColumns<WithIdentifier<char, T>>
	{
		self.scope(Self::DEFAULT_ALIAS)
	}

	/// Returns a [`RoleColumns`] which modifies its fields' [`Display`]
	/// implementation to output `{alias}.{column}`.
	///
	/// # See also
	///
	/// * [`WithIdentifier`]
	#[allow(clippy::missing_const_for_fn)]
	pub fn scope<Alias>(self, alias: Alias) -> RoleColumns<WithIdentifier<Alias, T>>
	where
		Alias: Copy,
	{
		RoleColumns {
			id: WithIdentifier(alias, self.id),
			name: WithIdentifier(alias, self.name),
			password_ttl: WithIdentifier(alias, self.password_ttl),
		}
	}
}

impl RoleColumns<&'static str>
{
	/// The names of the columns in `organizations` without any aliasing.
	pub const fn default() -> Self
	{
		Self { id: "id", name: "name", password_ttl: "password_ttl" }
	}
}
