//! Implementation of [`DateTimeExt`] for [`User`]

use winvoice_adapter_postgres::fmt::DateTimeExt;

use super::User;

impl DateTimeExt for User
{
	fn pg_sanitize(self) -> Self
	{
		Self { password_set: self.password_set.pg_sanitize(), ..self }
	}
}
