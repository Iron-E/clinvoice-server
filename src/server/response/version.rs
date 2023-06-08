//! Contains [`Version`] JSON from the [`winvoice_server::api`] which is a proper HTTP [`Response`].

use super::{Response, StatusCode};
use crate::api::{response::Version, Code, Status};

crate::new_response!(VersionResponse(Version): Clone, Debug, Default, Eq, Hash, PartialEq, Ord, PartialOrd);

impl VersionResponse
{
	/// Indicates there was an [encoding error](Code::EncodingError) while checking the supported
	/// version.
	pub fn encoding_err(message: String) -> Self
	{
		Self(Response::from(Version::new(Status::new(Code::EncodingError, message))))
	}

	/// Indicates the API version header was [mismatched](Code::ApiVersionMismatch).
	pub fn mismatch() -> Self
	{
		Self(Response::from(Version::new(Code::ApiVersionMismatch.into())))
	}

	/// Indicates the API version header was [missing](Code::ApiVersionHeaderMissing).
	pub fn missing() -> Self
	{
		Self(Response::from(Version::new(Code::ApiVersionHeaderMissing.into())))
	}

	/// Create a new [`VersionResponse`].
	pub const fn new(code: StatusCode, status: Status) -> Self
	{
		Self(Response::new(code, Version::new(status)))
	}
}
