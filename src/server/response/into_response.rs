use axum::{http::StatusCode, response::IntoResponse, Json};

use super::Response;

impl<T> IntoResponse for Response<T>
where
	(StatusCode, Json<T>): IntoResponse,
{
	fn into_response(self) -> axum::response::Response
	{
		(self.0, self.1).into_response()
	}
}
