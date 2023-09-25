//! Contains the [`From`] implementations for [`Status`].

use argon2::password_hash::Error as HashError;
use casbin::Error as CasbinError;
use money2::Error as MoneyError;
use sqlx::Error as SqlxError;
use winvoice_schema::chrono::OutOfRangeError;

use super::{Code, Status};
use crate::{
	permissions::{Action, Object},
	schema::User,
};

impl From<Code> for Status
{
	fn from(code: Code) -> Self
	{
		Self::new(code, code.to_string())
	}
}

impl From<&CasbinError> for Status
{
	fn from(error: &CasbinError) -> Self
	{
		Self::new(error.into(), error.to_string())
	}
}

impl From<&HashError> for Status
{
	fn from(error: &HashError) -> Self
	{
		Self::new(error.into(), error.to_string())
	}
}

impl From<&MoneyError> for Status
{
	fn from(error: &MoneyError) -> Self
	{
		Self::new(error.into(), error.to_string())
	}
}

impl From<&OutOfRangeError> for Status
{
	fn from(error: &OutOfRangeError) -> Self
	{
		Self::new(error.into(), error.to_string())
	}
}

impl From<&SqlxError> for Status
{
	fn from(error: &SqlxError) -> Self
	{
		Self::new(error.into(), error.to_string())
	}
}

impl From<(&User, Object, Action)> for Status
{
	/// Creates a status message declaring that this [`User`] did not have permission to perform
	/// some [`Action`] on an [`Object`].
	fn from(enforce: (&User, Object, Action)) -> Self
	{
		Self::new(
			Code::Unauthorized,
			format!("{} is not authorized to {} {}", enforce.0.username(), enforce.2, enforce.1),
		)
	}
}
