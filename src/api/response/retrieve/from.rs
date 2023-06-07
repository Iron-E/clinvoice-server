//! Implementations of [`From`] for [`Retrieve`].

use super::{Retrieve, Status};

impl<T> From<Status> for Retrieve<T>
{
	fn from(status: Status) -> Self
	{
		Self::new(Vec::<T>::default(), status)
	}
}
