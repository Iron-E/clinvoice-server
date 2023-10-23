//! Contains [`Login`] JSON from the [`winvoice_server::api`] which is a proper HTTP [`Response`].

mod from;

use winvoice_schema::chrono::{DateTime, Utc};

use super::{Response, StatusCode};
use crate::{
	api::{response::Login, Code, Status},
	schema::User,
};

crate::new_response!(LoginResponse(Login): Clone, Debug, Default, Eq, Hash, PartialEq, Ord, PartialOrd);

impl LoginResponse
{
	/// A [`LoginResponse`] indicating that the credentials passed were invalid.
	pub fn expired(date: DateTime<Utc>) -> Self
	{
		const CODE: Code = Code::PasswordExpired;
		Self::new(CODE.into(), Status::new(CODE, format!("Password expired on {date}")), None)
	}

	/// A [`LoginResponse`] indicating that the credentials passed were invalid.
	pub fn invalid_credentials(message: Option<String>) -> Self
	{
		const CODE: Code = Code::InvalidCredentials;
		Self::new(CODE.into(), message.map_or_else(|| CODE.into(), |m| Status::new(CODE, m)), None)
	}

	/// Create a new [`LoginResponse`].
	pub const fn new(code: StatusCode, status: Status, user: Option<User>) -> Self
	{
		Self(Response::new(code, Login::new(status, user)))
	}
}
