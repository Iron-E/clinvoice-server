use core::fmt::{Display, Formatter, Result};

use super::Reason;

impl Display for Reason
{
	fn fmt(&self, f: &mut Formatter<'_>) -> Result
	{
		match self
		{
			Self::NoDepartment => "they have no employee record to have been assigned a department with",
			Self::NoEmployee => "they have no employee record",
			Self::NoResourceExists => "no such resource exists",
            Self::ResourceConstraint => "another resource depends on it",
			Self::ResourceExists => "that resource already exists",
		}
		.fmt(f)
	}
}
