//! This module contains the template for a response which is sent by the [`winvoice_server`]

mod debug;
mod hash;
mod into_response;
mod login_response;
mod logout_response;
mod partial_eq;
mod partial_ord;

use axum::{http::StatusCode, Json};
pub use login_response::LoginResponse;
pub use logout_response::LogoutResponse;

use crate::api::Code;

/// Implements [`IntoResponse`](axum::response::IntoResponse) for any `struct` with this structure:
///
/// ```rust,ignore
/// struct Foo(T); // where `T` implements `IntoResponse`
/// impl_into_response!(Foo);
/// ```
#[macro_export]
macro_rules! new_response {
	($Name:ident($Type:ty) $(: $($derive:ident),+)*) => {
		#[doc = concat!(" A [`", stringify!($Type), "`] [`Response`](crate::server::Response)")]
		$(#[derive($($derive),+)])*
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

/// The response which the [`winvoice_server`] may issue.
#[derive(Clone, Copy, Default)]
pub struct Response<T>(StatusCode, Json<T>);

impl<T> Response<T>
{
	/// Create a new [`Response`]
	pub const fn new(status_code: StatusCode, content: T) -> Self
	{
		Self(status_code, Json(content))
	}
}

impl<T> Response<T>
where
	T: AsRef<Code>,
{
	/// Create a new [`Response`]
	pub fn from(content: T) -> Self
	{
		Self((*content.as_ref()).into(), Json(content))
	}
}
