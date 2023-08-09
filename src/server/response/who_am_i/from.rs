//! Contains [`From`] implementations for a [`WhoAmIResponse`].

use axum::http::StatusCode;

use super::{WhoAmI, WhoAmIResponse};
use crate::server::response::Response;

impl From<String> for WhoAmIResponse
{
	fn from(s: String) -> Self
	{
		Self::from(Response::new(StatusCode::OK, WhoAmI::new(s)))
	}
}
