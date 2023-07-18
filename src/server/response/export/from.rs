//! Contains [`From`] implementations for a [`ExportResponse`].

use std::collections::HashMap;

use money2::Error as MoneyError;
use sqlx::Error as SqlxError;

use super::ExportResponse;
use crate::api::{Code, Status};

impl From<Code> for ExportResponse
{
	fn from(code: Code) -> Self
	{
		Self::from(Status::from(code))
	}
}

impl From<MoneyError> for ExportResponse
{
	fn from(error: MoneyError) -> Self
	{
		Self::from(Status::from(&error))
	}
}

impl From<SqlxError> for ExportResponse
{
	fn from(error: SqlxError) -> Self
	{
		Self::from(Status::from(&error))
	}
}

impl From<Status> for ExportResponse
{
	fn from(status: Status) -> Self
	{
		Self::new(status.code().into(), Default::default(), status)
	}
}

impl From<HashMap<String, String>> for ExportResponse
{
	fn from(exported: HashMap<String, String>) -> Self
	{
		const CODE: Code = Code::Success;
		Self::new(CODE.into(), exported, CODE.into())
	}
}
