//! Implementations of [`From`] for [`Retrieve`].

use super::{Get, Status};

impl<T> From<Status> for Get<T>
{
	fn from(status: Status) -> Self
	{
		Self::new(Vec::<T>::default(), status)
	}
}
