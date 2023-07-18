//! Implementations of [`From`] for [`Retrieve`].

use super::{Export, Status};

impl From<Status> for Export
{
	fn from(status: Status) -> Self
	{
		Self::new(Default::default(), status)
	}
}
