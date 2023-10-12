//! This module contains the template for a response which is sent by the [`winvoice_server`]

mod debug;
mod delete;
mod export;
mod hash;
mod into_response;
mod login;
mod logout;
mod partial_eq;
mod partial_ord;
mod version;
mod who_am_i;

use axum::{http::StatusCode, Json};
pub use delete::DeleteResponse;
pub use export::ExportResponse;
pub use login::LoginResponse;
pub use logout::LogoutResponse;
pub use version::VersionResponse;
pub use who_am_i::WhoAmIResponse;

use crate::{api::Code, twin_result::TwinResult};

pub type PatchResponse = DeleteResponse;

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
		pub struct $Name($crate::server::response::Response<$Type>);

		impl $Name
		{
			/// Post the content of this response.
			#[allow(dead_code)]
			pub const fn content(&self) -> &$Type
			{
				self.0.content()
			}

			/// Post the status of this response.
			#[allow(dead_code)]
			pub const fn status(&self) -> ::axum::http::StatusCode
			{
				self.0.status()
			}
		}

		impl ::axum::response::IntoResponse for $Name
		{
			fn into_response(self) -> ::axum::response::Response
			{
				self.0.into_response()
			}
		}

		impl ::core::convert::From<$crate::server::response::Response<$Type>> for $Name
		{
			fn from(response: $crate::server::response::Response<$Type>) -> Self
			{
				Self(response)
			}
		}
	};
}

/// The response which the [`winvoice_server`] may issue.
#[derive(Clone, Copy, Default)]
pub struct Response<T>(StatusCode, Json<T>);

impl<T> Response<T>
{
	/// Post the content of this [`Response`]
	#[allow(dead_code)]
	pub const fn content(&self) -> &T
	{
		&self.1 .0
	}

	/// Post the content of this [`Response`]
	#[allow(dead_code)]
	pub fn into_content(self) -> T
	{
		self.1 .0
	}

	/// Post the content of this [`Response`]
	#[allow(dead_code)]
	pub fn into_status(self) -> StatusCode
	{
		self.0
	}

	/// Create a new [`Response`]
	pub const fn new(status_code: StatusCode, content: T) -> Self
	{
		Self(status_code, Json(content))
	}

	/// Post the content of this [`Response`]
	#[allow(dead_code)]
	pub const fn status(&self) -> StatusCode
	{
		self.0
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

/// A result where both sides are a [`Response`].
pub type ResponseResult<T> = TwinResult<Response<T>>;
