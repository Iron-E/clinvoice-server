//! Contains an implementation of [`AuthUser`] for [`User`].

use axum_login::{secrecy::SecretVec, AuthUser};

use super::User;

impl AuthUser<String, String> for User
{
	fn get_id(&self) -> String
	{
		self.username.clone()
	}

	fn get_password_hash(&self) -> SecretVec<u8>
	{
		SecretVec::new(Vec::from(self.password()))
	}

	fn get_role(&self) -> Option<String>
	{
		Some(self.role.clone())
	}
}
