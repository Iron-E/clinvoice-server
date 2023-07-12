//! An implementation of [`Display`] for [`Object`]

use core::fmt::{Display, Formatter, Result};

use super::Object;

impl Display for Object
{
	fn fmt(&self, f: &mut Formatter<'_>) -> Result
	{
		match self
		{
			Self::AssignedDepartment => "the department they were assigned to",
			Self::Contact => "contacts",
			Self::CreatedExpenses => "their created expenses",
			Self::CreatedTimesheet => "their created timesheets",
			Self::Department => "departments",
			Self::Employee => "employees",
			Self::EmployeeInDepartment => "employees in their department",
			Self::Expenses => "expenses",
			Self::ExpensesInDepartment => "expenses in their department",
			Self::Job => "jobs",
			Self::JobInDepartment => "jobs in their department",
			Self::Location => "locations",
			Self::Organization => "organization",
			Self::Role => "roles",
			Self::Timesheet => "timesheets",
			Self::TimesheetInDepartment => "timesheets in their department",
			Self::User => "users",
			Self::UserInDepartment => "users in their department",
		}
		.fmt(f)
	}
}
