//! An implementation of [`Display`] for [`Action`]

use core::fmt::{Display, Formatter, Result};

use super::Action;

impl Display for Action
{
	fn fmt(&self, f: &mut Formatter<'_>) -> Result
	{
		match self
		{
			Self::Create => "create",
			Self::Delete => "delete",
			Self::Retrieve => "retrieve",
			Self::Update => "update",
		}
		.fmt(f)
	}
}
