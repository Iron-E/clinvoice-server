//! This module holds data for the columns of the [`User`](crate::api::schema::User) table.

mod columns_to_sql;
mod table_to_sql;

use serde::{Deserialize, Serialize};
use winvoice_adapter::fmt::{TableToSql, WithIdentifier};

/// The names of the columns of the `timesheets` table.
#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct UserColumns<T>
{
	/// The name of the `employee_id` column of the `timesheets` table.
	pub employee_id: T,

	/// The name of the `id` column of the `timesheets` table.
	pub id: T,
}

impl<T> UserColumns<T>
{
	/// Add a [scope](ExpenseColumns::scope) using the [default alias](TableToSql::default_alias)
	///
	/// # See also
	///
	/// * [`WithIdentifier`].
	pub fn default_scope(self) -> UserColumns<WithIdentifier<char, T>>
	{
		self.scope(Self::DEFAULT_ALIAS)
	}

	/// Returns a [`UserColumns`] which modifies its fields' [`Display`]
	/// implementation to output `{alias}.{column}`.
	///
	/// # See also
	///
	/// * [`WithIdentifier`]
	#[allow(clippy::missing_const_for_fn)]
	pub fn scope<Alias>(self, alias: Alias) -> UserColumns<WithIdentifier<Alias, T>>
	where
		Alias: Copy,
	{
		UserColumns {
			employee_id: WithIdentifier(alias, self.employee_id),
			id: WithIdentifier(alias, self.id),
			job_id: WithIdentifier(alias, self.job_id),
			time_begin: WithIdentifier(alias, self.time_begin),
			time_end: WithIdentifier(alias, self.time_end),
			work_notes: WithIdentifier(alias, self.work_notes),
		}
	}
}

impl UserColumns<&'static str>
{
	/// The names of the columns in `organizations` without any aliasing.
	pub const fn default() -> Self
	{
		Self {
			id: "id",
			employee_id: "employee_id",
			job_id: "job_id",
			time_begin: "time_begin",
			time_end: "time_end",
			work_notes: "work_notes",
		}
	}
}
