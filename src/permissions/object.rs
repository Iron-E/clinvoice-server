//! Contains all the actions which may be taken by users.

mod display;

use serde::{Deserialize, Serialize};

/// The actions which users may have permission to take.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Object
{
	/// Permission to operate the [`winvoice_schema::Department`] which a
	/// [`User`](crate::schema::User) was assigned to.
	AssignedDepartment,

	/// Permission to operate on [`winvoice_schema::Contact`]s
	Contact,

	/// Permission to operate on [`winvoice_schema::Expense`]s which are associated with
	/// [`winvoice_schema::Timesheet`]s that were created by the [`User`](crate::schema::User) who
	/// has been given this permission.
	///
	/// The [`Create`](super::Action::Create) permission means nothing on this object type.
	#[serde(rename = "created_expense")]
	CreatedExpenses,

	/// Permission to operate on [`winvoice_schema::Timesheet`]s which that particular
	/// [`User`](crate::schema::User) has created.
	///
	/// The [`Create`](super::Action::Create) permission means nothing on this object type.
	CreatedTimesheet,

	/// Permission to operate on [`winvoice_schema::Department`]s. Assumes
	/// [`AssignedDepartment`](Self::AssignedDepartment).
	Department,

	/// Permission to operate on [`winvoice_schema::Employee`]s. Assumes
	/// [`EmployeeInDepartment`](Self::EmployeeInDepartment).
	Employee,

	/// Permission to act on one's own employee record.
	#[serde(skip)]
	EmployeeSelf,

	/// Permission to operate on [`winvoice_schema::Employee`]s which are in a given [`User`]'s
	/// assigned [`Department`](winvoice_schema::Department).
	EmployeeInDepartment,

	/// Permission to operate on [`winvoice_schema::Expense`]s. Assumes
	/// [`CreatedExpenses`](Self::CreatedExpenses)
	#[serde(rename = "expense")]
	Expenses,

	/// Permission to operate on [`winvoice_schema::Expense`]s. Assumes
	/// [`CreatedExpenses`](Self::CreatedExpenses)
	#[serde(rename = "expense_in_department")]
	ExpensesInDepartment,

	/// Permission to operate on [`winvoice_schema::Job`]s. Assumes
	/// [`JobInDepartment`](Self::JobInDepartment`].
	Job,

	/// Permission to operate on [`winvoice_schema::Job`]s in the [`User`](crate::schema::User)'s
	/// [assigned department](Self::AssignedDepartment).
	JobInDepartment,

	/// Permission to operate on [`winvoice_schema::Location`]s
	Location,

	/// Permission to operate on [`winvoice_schema::Organization`]s
	Organization,

	/// Permission to operate on [`Role`](crate::schema::Role)s
	Role,

	/// Permission to operate on [`winvoice_schema::Timesheet`]s. Assumes
	/// [`CreatedTimesheet`](Self::CreatedTimesheet).
	Timesheet,

	/// Permission to operate on [`winvoice_schema::Timesheet`]s. Assumes
	/// [`CreatedTimesheet`](Self::CreatedTimesheet).
	TimesheetInDepartment,

	/// Permission to operate on [`User`](crate::schema::User)s
	User,

	/// Permission to operate on [`User`](crate::schema::User)s
	UserInDepartment,

	/// Permission to act on one's own user.
	#[serde(skip)]
	UserSelf,
}

impl Object
{
	/// Denote the given [`Object`] as an impossible match on a given match arm.
	///
	/// ```rust
	/// use winvoice_server::permissions::Object;
	/// match Object::Employee {
	///   Object::Employee => println!("matched"),
	///   o => o.unreachable(),
	/// };
	/// ```
	///
	/// ```rust,should_panic
	/// # use winvoice_server::permissions::Object;
	/// match Object::Employee {
	///   Object::Department => println!("matched"),
	///   o => o.unreachable(), // panics
	/// };
	/// ```
	pub fn unreachable(&self) -> !
	{
		unreachable!("unexpected permission: {self:?}")
	}
}
