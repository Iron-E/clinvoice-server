//! A [`Debug`] impl for [`Response`].

use core::fmt::{Debug, Formatter, Result};

use super::Response;

impl<T> Debug for Response<T>
where
	T: Debug,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> Result
	{
		f.debug_tuple("Response").field(&self.0).field(&self.1).finish()
	}
}
