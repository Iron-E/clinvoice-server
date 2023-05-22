//! This module contains the template for a response which is sent by the [`winvoice-server`](crate)

mod into_response;
mod login_response;
mod logout_response;

use axum::{http::StatusCode, Json};
pub use login_response::LoginResponse;
pub use logout_response::LogoutResponse;

/// Implements [`IntoResponse`](axum::response::IntoResponse) for any `struct` with this structure:
///
/// ```rust,ignore
/// struct Foo(T); // where `T` implements `IntoResponse`
/// impl_into_response!(Foo);
/// ```
#[macro_export]
macro_rules! new_response {
	($Name:ident, $Type:ty) => {
		#[doc = concat!(" A [`", stringify!($Type), "`] [`Response`](crate::server::Response)")]
		pub struct $Name($crate::server::Response<$Type>);

		impl ::axum::response::IntoResponse for $Name
		{
			fn into_response(self) -> ::axum::response::Response
			{
				self.0.into_response()
			}
		}
	};
}

/// The response which the [`winvoice-server`](crate) may issue.
pub struct Response<T>(StatusCode, Json<T>);

impl<T> Response<T>
{
	/// Create a new [`Response`]
	pub const fn new(status_code: StatusCode, content: T) -> Self
	{
		Self(status_code, Json(content))
	}
}
