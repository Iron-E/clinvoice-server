//! Contains [`From`] implementations for [`MatchUser`].

use super::{Id, Match, MatchUser};
use crate::schema::User;

impl From<Id> for MatchUser
{
	fn from(id: Id) -> Self
	{
		Match::from(id).into()
	}
}

impl From<Match<Id>> for MatchUser
{
	fn from(match_condition: Match<Id>) -> Self
	{
		Self { id: match_condition, ..Default::default() }
	}
}

impl From<User> for MatchUser
{
	fn from(user: User) -> Self
	{
		Self {
			id: user.id().into(),
			password_set: user.password_set().into(),
			employee: user.employee.map(Into::into).into(),
			password: user.password.into(),
			role: user.role.into(),
			username: user.username.into(),
		}
	}
}
