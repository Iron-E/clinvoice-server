//! Implementations of [`From`] for [`Retrieve`].

use super::{Put, Status};

impl<T> From<Status> for Put<T>
{
	fn from(status: Status) -> Self
	{
		Self::new(None, status)
	}
}
