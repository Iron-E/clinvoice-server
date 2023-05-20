//! This module contains the template for a response which is sent by the [`winvoice-server`](crate)

mod into_response;

use axum::{http::StatusCode, Json};

/// The response which the [`winvoice-server`](crate) may issue.
pub struct Response<T>(StatusCode, Json<T>);

impl<T> Response<T>
{
	/// Create a new [`Response`]
	pub fn new(status_code: StatusCode, content: T) -> Self
	{
		Response(status_code, Json(content))
	}
}
