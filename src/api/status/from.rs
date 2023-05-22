//! Contains the [`From`] implementations for [`Status`].

use base64_url::base64::DecodeError;

use super::{Code, Status};
use crate::api::TokenParseError;

impl From<&DecodeError> for Status
{
	fn from(error: &DecodeError) -> Self
	{
		Self::new(Code::DecodeError, error.to_string())
	}
}

impl From<&sqlx::Error> for Status
{
	fn from(error: &sqlx::Error) -> Self
	{
		use sqlx::Error;

		Self::new(
			match error
			{
				Error::Configuration(_) => Code::BadArguments,
				#[cfg(feature = "postgres")]
				Error::Database(e)
					if matches!(
						e.try_downcast_ref::<sqlx::postgres::PgDatabaseError>()
							.and_then(sqlx::postgres::PgDatabaseError::routine),
						Some("auth_failed" | "InitializeSessionUserId"),
					) =>
				{
					Code::InvalidCredentials
				},
				Error::ColumnDecode { .. } | Error::Decode(_) => Code::DecodeError,
				Error::ColumnIndexOutOfBounds { .. } |
				Error::ColumnNotFound(_) |
				Error::RowNotFound |
				Error::TypeNotFound { .. } => Code::SqlError,
				Error::Io(_) => Code::DbIoError,
				Error::PoolClosed => Code::DbConnectionSevered,
				Error::PoolTimedOut => Code::DbConnectTimeout,
				Error::Protocol(_) => Code::DbAdapterError,
				Error::Tls(_) => Code::DbTlsError,
				_ => Code::Other,
			},
			error.to_string(),
		)
	}
}

impl From<&TokenParseError> for Status
{
	fn from(error: &TokenParseError) -> Self
	{
		Self::new(Code::DecodeError, error.to_string())
	}
}
