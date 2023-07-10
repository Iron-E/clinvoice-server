//! Contains [`Delete`] JSON from the [`winvoice_server::api`] which is a proper HTTP [`Response`].

mod from;

use super::{Response, StatusCode};
use crate::api::{response::Delete, Status};

crate::new_response!(DeleteResponse(Delete): Clone, Debug, Default, Eq, Hash, PartialEq, Ord, PartialOrd);

impl DeleteResponse
{
	/// Create a new [`DeleteResponse`].
	pub const fn new(code: StatusCode, status: Status) -> Self
	{
		Self(Response::new(code, Delete::new(status)))
	}
}
