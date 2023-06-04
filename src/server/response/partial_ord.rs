//! A [`PartialOrd`] impl for [`Response`].

use core::cmp::Ordering;

use super::Response;

impl<T> PartialOrd for Response<T>
where
	T: PartialOrd,
{
	fn partial_cmp(&self, other: &Self) -> Option<Ordering>
	{
		let cmp_1 = || self.1 .0.partial_cmp(&self.1 .0);
		self.0.partial_cmp(&other.0).map(|c| c.then_with(|| cmp_1().unwrap_or(c))).or_else(cmp_1)
	}
}

impl<T> Ord for Response<T>
where
	T: Ord,
{
	fn cmp(&self, other: &Self) -> Ordering
	{
		self.0.cmp(&other.0).then_with(|| self.1 .0.cmp(&self.1 .0))
	}
}
