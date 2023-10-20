//! Contains [`From`] implementations for a [`WhoAmIResponse`].

use axum::http::StatusCode;

use super::{WhoAmI, WhoAmIResponse};
use crate::{schema::User, server::response::Response};

impl From<User> for WhoAmIResponse
{
	fn from(u: User) -> Self
	{
		Self::from(Response::new(StatusCode::OK, WhoAmI::new(u)))
	}
}
