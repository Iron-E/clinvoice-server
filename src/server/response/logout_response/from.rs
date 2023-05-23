//! Contains [`From`] implementations for a [`LoginResponse`].

use core::array::TryFromSliceError;

use axum::http::StatusCode;

use super::LogoutResponse;
use crate::api::Status;

impl From<&TryFromSliceError> for LogoutResponse
{
	fn from(error: &TryFromSliceError) -> Self
	{
		Self::new(StatusCode::BAD_REQUEST, Status::from(error))
	}
}
