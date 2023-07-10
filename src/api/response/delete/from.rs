//! Implementations of [`From`] for [`Retrieve`].

use super::{Delete, Status};

impl From<Status> for Delete
{
	fn from(status: Status) -> Self
	{
		Self::new(status)
	}
}
