//! A [`Hash`] impl for [`Response`].

use core::hash::{Hash, Hasher};

use super::Response;

impl<T> Hash for Response<T>
where
	T: Hash,
{
	fn hash<H>(&self, state: &mut H)
	where
		H: Hasher,
	{
		self.0.hash(&mut *state);
		self.1 .0.hash(state);
	}
}
