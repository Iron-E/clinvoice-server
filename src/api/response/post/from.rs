//! Implementations of [`From`] for [`Retrieve`].

use super::{Post, Status};

impl<T> From<Status> for Post<T>
{
	fn from(status: Status) -> Self
	{
		Self::new(None, status)
	}
}
