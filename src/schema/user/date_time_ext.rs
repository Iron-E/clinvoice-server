//! Implementation of [`DateTimeExt`] for [`User`]

use winvoice_adapter_postgres::fmt::DateTimeExt;

use super::User;

impl DateTimeExt for User
{
	fn pg_sanitize(self) -> Self
	{
		Self { password_expires: self.password_expires.map(DateTimeExt::pg_sanitize), ..self }
	}
}
