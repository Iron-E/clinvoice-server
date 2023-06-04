//! Contains [`From`] implementations for a [`LoginResponse`].

use axum::http::StatusCode;
use sqlx::Error;

use super::LoginResponse;
use crate::api::{Code, Status};

impl From<&Error> for LoginResponse
{
	fn from(error: &Error) -> Self
	{
		let status = Status::from(error);
		Self::new(
			match status.code()
			{
				Code::InvalidCredentials => StatusCode::UNPROCESSABLE_ENTITY,
				_ => StatusCode::INTERNAL_SERVER_ERROR,
			},
			status,
		)
	}
}
