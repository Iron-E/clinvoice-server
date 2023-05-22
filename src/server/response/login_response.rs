//! Contains [`Login`] JSON from the [`winvoice_server::api`] which is a proper HTTP [`Response`].

mod from;

use super::{Response, StatusCode};
use crate::api::{response::Login, Status, StatusCode as Code};

crate::new_response!(LoginResponse, Login);

impl LoginResponse
{
	/// Create a new [`LoginResponse`].
	pub fn new<S>(code: StatusCode, status: S) -> Self
	where
		S: Into<Status>,
	{
		Self(Response::new(code, Login::new(status.into())))
	}

	pub fn success() -> Self
	{
		Self::new(StatusCode::OK, Status::new(Code::LoggedIn, None))
	}
}
