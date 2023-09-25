//! Implementations of [`From`] for [`Code`].

use argon2::password_hash::Error as PasswordError;
use axum::http::StatusCode;
use casbin::Error as CasbinError;
use money2::Error as MoneyError;
use sqlx::Error as SqlxError;
use winvoice_schema::chrono::OutOfRangeError;

use super::Code;

impl From<Code> for u8
{
	fn from(code: Code) -> Self
	{
		code as Self
	}
}

impl From<Code> for StatusCode
{
	fn from(code: Code) -> Self
	{
		match code
		{
			Code::ApiVersionMismatch => Self::GONE,
			Code::InvalidCredentials | Code::PasswordExpired => Self::UNPROCESSABLE_ENTITY,
			Code::Success | Code::SuccessForPermissions => Self::OK,
			Code::Unauthorized => Self::FORBIDDEN,

			Code::ApiVersionHeaderMissing | Code::EncodingError => Self::BAD_REQUEST,

			Code::BadArguments |
			Code::CryptError |
			Code::Database |
			Code::ExchangeError |
			Code::LoginError |
			Code::Other |
			Code::PermissionsError |
			Code::SqlError => Self::INTERNAL_SERVER_ERROR,
		}
	}
}

impl From<&CasbinError> for Code
{
	fn from(error: &CasbinError) -> Self
	{
		match error
		{
			CasbinError::RequestError(_) => Self::PermissionsError,

			CasbinError::AdapterError(_) |
			CasbinError::IoError(_) |
			CasbinError::ModelError(_) |
			CasbinError::PolicyError(_) |
			CasbinError::RbacError(_) |
			CasbinError::RhaiError(_) |
			CasbinError::RhaiParseError(_) => Self::Other,
		}
	}
}

impl From<&MoneyError> for Code
{
	fn from(_: &MoneyError) -> Self
	{
		Self::ExchangeError
	}
}

impl From<&OutOfRangeError> for Code
{
	fn from(_: &OutOfRangeError) -> Self
	{
		Self::EncodingError
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

			SqlxError::ColumnIndexOutOfBounds { .. } |
			SqlxError::ColumnNotFound(_) |
			SqlxError::RowNotFound |
			SqlxError::TypeNotFound { .. } => Self::SqlError,

			SqlxError::ColumnDecode { .. } |
			SqlxError::Decode(_) |
			SqlxError::Io(_) |
			SqlxError::PoolClosed |
			SqlxError::PoolTimedOut |
			SqlxError::Protocol(_) |
			SqlxError::Tls(_) => Self::Database,

			_ => Self::Other,
		}
	}
}
