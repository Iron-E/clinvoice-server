//! Contains [`Login`] JSON from the [`winvoice_server::api`] which is a proper HTTP [`Response`].

mod from;

use super::{Response, StatusCode};
use crate::api::{response::Login, Code, Status};

crate::new_response!(LoginResponse, Login);

impl LoginResponse
{
	/// A [`LoginResponse`] indicating that the credentials passed were invalid.
	pub const fn invalid_credentials() -> Self
	{
		Self::new(StatusCode::UNPROCESSABLE_ENTITY, Code::InvalidCredentials.into())
	}

	/// Create a new [`LoginResponse`].
	pub fn new(code: StatusCode, status: Status) -> Self
	{
		Self(Response::new(code, Login::new(status)))
	}

	/// A [`LoginResponse`] indicating the login operation succeeded.
	pub const fn success() -> Self
	{
		Self::new(StatusCode::OK, Code::LoggedIn.into())
	}
}
