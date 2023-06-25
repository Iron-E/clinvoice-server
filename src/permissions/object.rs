//! Contains all the actions which may be taken by users.

mod display;

use serde::{Deserialize, Serialize};

/// The actions which users may have permission to take.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Object
{
	/// Permission to operate on [`winvoice_schema::Contact`]s
	Contact,

	/// Permission to operate on [`winvoice_schema::Contact`]s
	Department,

	/// Permission to operate on [`winvoice_schema::Employee`]s
	Employee,

	/// Permission to operate on [`winvoice_schema::Expense`]s
	#[serde(rename = "expense")]
	Expenses,

	/// Permission to operate on [`winvoice_schema::Job`]s
	Job,

	/// Permission to operate on [`winvoice_schema::Location`]s
	Location,

	/// Permission to operate on [`winvoice_schema::Organization`]s
	Organization,

	/// Permission to operate on [`Role`](crate::schema::Role)s
	Role,

	/// Permission to operate on [`winvoice_schema::Timesheet`]s
	Timesheet,

	/// Permission to operate on [`User`](crate::schema::User)s
	User,
}
