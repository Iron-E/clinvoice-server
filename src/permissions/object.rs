//! Contains all the actions which may be taken by users.

use serde::{Deserialize, Serialize};

/// The actions which users may have permission to take.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Object
{
	/// Permissions for operating on [`winvoice_schema::Contact`]s
	Contact,

	/// Permissions for operating on [`winvoice_schema::Employee`]s
	Employee,

	/// Permissions for operating on [`winvoice_schema::Expense`]s
	Expense,

	/// Permissions for operating on [`winvoice_schema::Job`]s
	Job,

	/// Permissions for operating on [`winvoice_schema::Location`]s
	Location,

	/// Permissions for operating on [`winvoice_schema::Organization`]s
	Organization,

	/// Permissions for operating on [`winvoice_schema::Timesheet`]s
	Timesheet,
}
