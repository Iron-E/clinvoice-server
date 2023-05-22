//! Contains the [`From`] implementations for [`Status`].

use sqlx::Error;

use super::{Code, Status};

impl From<&Error> for Status
{
	fn from(error: &Error) -> Self
	{
		match error
		{
			Error::Configuration(e) => Self::new(Code::BadArguments, e.to_string()),
			#[cfg(feature = "postgres")]
			Error::Database(e)
				if matches!(
					e.try_downcast_ref::<sqlx::postgres::PgDatabaseError>()
						.and_then(sqlx::postgres::PgDatabaseError::routine),
					Some("auth_failed" | "InitializeSessionUserId"),
				) =>
			{
				Self::new(Code::InvalidCredentials, e.to_string())
			},
			Error::Io(e) => Self::new(Code::DatabaseIoError, e.to_string()),
			_ => Self::new(Code::Other, error.to_string()),
		}
	}
}
