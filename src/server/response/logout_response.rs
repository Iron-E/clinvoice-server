//! Contains [`Logout`] JSON from the [`winvoice_server::api`] which is a proper HTTP [`Response`].

mod from;

use super::{Response, StatusCode};
use crate::api::{response::Logout, Code, Status};

crate::new_response!(LogoutResponse(Logout): Clone, Default, Eq, Hash, PartialEq, Ord, PartialOrd);

impl LogoutResponse
{
	/// Create a new [`LogoutResponse`].
	pub fn new(code: StatusCode, status: Status) -> Self
	{
		Self(Response::new(code, Logout::new(status)))
	}

	pub fn success() -> Self
	{
		Self::new(StatusCode::OK, Code::LoggedOut.into())
	}
}
