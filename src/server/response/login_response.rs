//! Contains [`Login`] JSON from the [`winvoice_server::api`] which is a proper HTTP [`Response`].

mod from;

use super::{Response, StatusCode};
use crate::api::{response::Login, Status};

crate::new_response!(LoginResponse, Login);

impl LoginResponse
{
	/// Create a new [`LoginResponse`].
	pub fn new(code: StatusCode, status: Status) -> Self
	{
		Self(Response::new(code, Login::new(status)))
	}
}
