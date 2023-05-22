//! Contains [`Logout`] JSON from the [`winvoice_server::api`] which is a proper HTTP [`Response`].

mod from;

use super::{Response, StatusCode};
use crate::api::{response::Logout, Status, StatusCode as Code};

crate::new_response!(LogoutResponse, Logout);

impl LogoutResponse
{
	/// Create a new [`LogoutResponse`].
	pub fn new<S>(code: StatusCode, status: S) -> Self
	where
		S: Into<Status>,
	{
		Self(Response::new(code, Logout::new(status.into())))
	}

	pub fn success() -> Self
	{
		Self::new(StatusCode::OK, Status::new(Code::LoggedOut, None))
	}
}
