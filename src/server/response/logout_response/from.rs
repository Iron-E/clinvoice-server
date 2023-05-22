//! Contains [`From`] implementations for a [`LoginResponse`].

use axum::http::StatusCode;
use base64_url::base64::DecodeError;

use super::LogoutResponse;
use crate::api::{Status, TokenParseError};

impl From<&DecodeError> for LogoutResponse
{
	fn from(error: &DecodeError) -> Self
	{
		Self::new(StatusCode::BAD_REQUEST, Status::from(error))
	}
}

impl From<&TokenParseError> for LogoutResponse
{
	fn from(error: &TokenParseError) -> Self
	{
		Self::new(StatusCode::BAD_REQUEST, Status::from(error))
	}
}
