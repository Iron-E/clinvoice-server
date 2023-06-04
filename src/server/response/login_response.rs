//! Contains [`Login`] JSON from the [`winvoice_server::api`] which is a proper HTTP [`Response`].

mod from;

use super::{Response, StatusCode};
use crate::api::{response::Login, Code, Status};

crate::new_response!(LoginResponse(Login): Clone, Default, Eq, Hash, PartialEq, Ord, PartialOrd);

impl LoginResponse
{
	/// A [`LoginResponse`] indicating that the credentials passed were invalid.
	pub fn invalid_credentials(message: Option<String>) -> Self
	{
		const CODE: Code = Code::InvalidCredentials;
		Self::new(CODE.into(), message.map_or_else(|| CODE.into(), |m| Status::new(CODE, m)))
	}

	/// Create a new [`LoginResponse`].
	pub fn new(code: StatusCode, status: Status) -> Self
	{
		Self(Response::new(code, Login::new(status)))
	}

	/// A [`LoginResponse`] indicating the login operation succeeded.
	pub fn success() -> Self
	{
		Code::LoggedIn.into()
	}
}
