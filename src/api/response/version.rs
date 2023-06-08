//! This module contains the response for a login.

mod as_ref;

use serde::{Deserialize, Serialize};

use crate::api::{Code, Status};

/// The login request response.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Version
{
	/// The [`Status`] of the login request.
	status: Status,
}

impl Version
{
	/// Indicates there was an [encoding error](Code::EncodingError) while checking the supported
	/// version.
	pub const fn encoding_err(message: String) -> Self
	{
		Self::new(Status::new(Code::EncodingError, message))
	}

	/// Indicates the API version header was [mismatched](Code::ApiVersionMismatch).
	pub fn mismatch() -> Self
	{
		Self::new(Code::ApiVersionMismatch.into())
	}

	/// Indicates the API version header was [missing](Code::ApiVersionHeaderMissing).
	pub fn missing() -> Self
	{
		Self::new(Code::ApiVersionHeaderMissing.into())
	}

	/// Create a new [`Version`] response.
	pub const fn new(status: Status) -> Self
	{
		Self { status }
	}

	/// The [`Status`] of the login request.
	pub const fn status(&self) -> &Status
	{
		&self.status
	}
}
