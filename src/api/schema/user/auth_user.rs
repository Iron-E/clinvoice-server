//! Contains an implementation of [`AuthUser`] for [`User`].

use axum_login::{secrecy::SecretVec, AuthUser};
use winvoice_schema::Id;

use super::User;

impl AuthUser<Id> for User
{
	fn get_id(&self) -> Id
	{
		self.id
	}

	fn get_password_hash(&self) -> SecretVec<u8>
	{
		SecretVec::new(Vec::from(self.password()))
	}
}
