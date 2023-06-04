//! A [`PartialEq`] impl for [`Response`].

use super::Response;

impl<T> PartialEq for Response<T>
where
	T: PartialEq,
{
	fn eq(&self, other: &Self) -> bool
	{
		self.0.eq(&other.0) && self.1.eq(&self.1)
	}
}

impl<T> Eq for Response<T> where T: Eq {}
