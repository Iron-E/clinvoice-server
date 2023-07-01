//! This module holds data for the columns of the [`Role`](crate::schema::Role) table.

mod columns_to_sql;
mod table_to_sql;

use serde::{Deserialize, Serialize};
use winvoice_adapter::fmt::{As, TableToSql, WithIdentifier};

/// The names of the columns of the `roles` table.
#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct RoleColumns<T = &'static str>
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
	/// Returns a [`RoleColumns`] which aliases the names of these [`RoleColumns`] with the
	/// `aliased` columns provided.
	///
	/// # See also
	///
	/// * [`As`]
	#[allow(clippy::missing_const_for_fn)] // destructor cannot be evaluated at compile-time
	pub fn r#as<Alias>(self, aliased: RoleColumns<Alias>) -> RoleColumns<As<T, Alias>>
	{
		RoleColumns {
			id: As(self.id, aliased.id),
			name: As(self.name, aliased.name),
			password_ttl: As(self.password_ttl, aliased.password_ttl),
		}
	}

	/// Add a [scope](Self::scope) using the [default alias](TableToSql::DEFAULT_ALIAS)
	///
	/// # See also
	///
	/// * [`WithIdentifier`].
	#[allow(clippy::missing_const_for_fn)] // destructor cannot be evaluated at compile-time
	pub fn default_scope(self) -> RoleColumns<WithIdentifier<char, T>>
	{
		self.scope(RoleColumns::DEFAULT_ALIAS)
	}

	/// Returns a [`RoleColumns`] which modifies its fields' [`Display`](core::fmt::Display)
	/// implementation to output `{alias}.{column}`.
	///
	/// # See also
	///
	/// * [`WithIdentifier`]
	#[allow(clippy::missing_const_for_fn)] // destructor cannot be evaluated at compile-time
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

	/// Aliases for the columns in `roles` which are guaranteed to be unique among other
	/// [`columns`](super)' `unique` aliases.
	///
	/// # Examples
	///
	/// ```rust
	/// # use pretty_assertions::assert_eq;
	/// use sqlx::{Execute, Postgres, QueryBuilder};
	/// use winvoice_adapter::fmt::{QueryBuilderExt, sql};
	/// use winvoice_server::schema::columns::*;
	///
	/// {
	///   let mut query = QueryBuilder::<Postgres>::new(sql::SELECT);
	///
	///   // `sqlx::Row::get` ignores scopes (e.g. `E.` in `E.id`) so `R.id` and `U.id` clobber each
	///   // other.
	///   assert_eq!(
	///     query
	///       .push_columns(&RoleColumns::default().default_scope())
	///       .push_more_columns(&UserColumns::default().default_scope())
	///       .prepare()
	///       .sql(),
	///     " SELECT R.id,R.name,R.password_ttl,\
	///         U.employee_id,U.id,U.password,U.password_expires,U.role_id,U.username;"
	///   );
	/// }
	///
	/// {
	///   let mut query = QueryBuilder::<Postgres>::new(sql::SELECT);
	///
	///   // no clobbering
	///   assert_eq!(
	///     query
	///       .push_columns(&UserColumns::default().default_scope())
	///       .push_more_columns(&RoleColumns::default().default_scope().r#as(RoleColumns::unique()))
	///       .prepare()
	///       .sql(),
	///     " SELECT U.employee_id,U.id,U.password,U.password_expires,U.role_id,U.username,\
	///         R.id AS unique_8_role_id,\
	///         R.name AS unique_8_role_name,\
	///         R.password_ttl AS unique_8_role_password_ttl;"
	///   );
	/// }
	/// ```
	pub const fn unique() -> Self
	{
		Self { id: "unique_8_role_id", name: "unique_8_role_name", password_ttl: "unique_8_role_password_ttl" }
	}
}
