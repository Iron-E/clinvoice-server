//! An implementation of [`Display`] for [`Object`]

use core::fmt::{Display, Formatter, Result};

use super::Object;

impl Display for Object
{
	fn fmt(&self, f: &mut Formatter<'_>) -> Result
	{
		match self
		{
			Self::Contact => "contact",
			Self::Department => "department",
			Self::Employee => "employee",
			Self::Expenses => "expense",
			Self::Job => "job",
			Self::Location => "location",
			Self::Organization => "organization",
			Self::Role => "role",
			Self::Timesheet => "timesheet",
			Self::User => "user",
		}
		.fmt(f)
	}
}
