//! Implementations of [`From`] for [`Code`].

use argon2::password_hash::Error as PasswordError;
use axum::http::StatusCode;
use sqlx::Error as SqlxError;

use super::Code;

impl From<Code> for StatusCode
{
	fn from(code: Code) -> Self
	{
		match code
		{
			Code::EncodingError => Self::BAD_REQUEST,
			Code::InvalidCredentials => Self::UNPROCESSABLE_ENTITY,
			Code::Success => Self::OK,
			Code::Unauthorized => Self::UNAUTHORIZED,

			Code::BadArguments |
			Code::CryptError |
			Code::Database |
			Code::LoginError |
			Code::Other |
			Code::SqlError => Self::INTERNAL_SERVER_ERROR,
		}
	}
}

impl From<&PasswordError> for Code
{
	fn from(error: &PasswordError) -> Self
	{
		match error
		{
			PasswordError::B64Encoding(_) => Self::EncodingError,
			PasswordError::Crypto => Self::CryptError,
			PasswordError::Password => Self::InvalidCredentials,
			_ => Self::Other,
		}
	}
}

impl From<&SqlxError> for Code
{
	fn from(error: &SqlxError) -> Self
	{
		match error
		{
			SqlxError::Configuration(_) => Self::BadArguments,
			SqlxError::ColumnDecode { .. } | SqlxError::Decode(_) => Self::EncodingError,

			SqlxError::ColumnIndexOutOfBounds { .. } |
			SqlxError::ColumnNotFound(_) |
			SqlxError::RowNotFound |
			SqlxError::TypeNotFound { .. } => Self::SqlError,

			SqlxError::Io(_) |
			SqlxError::PoolClosed |
			SqlxError::PoolTimedOut |
			SqlxError::Protocol(_) |
			SqlxError::Tls(_) => Self::Database,

			_ => Self::Other,
		}
	}
}
