//! Contains [`From`] implementations for [`MatchRole`].

use humantime_serde::Serde;

use super::{Id, Match, MatchRole, MatchStr};
use crate::schema::Role;

impl From<Id> for MatchRole
{
	fn from(id: Id) -> Self
	{
		Match::from(id).into()
	}
}

impl From<Match<Id>> for MatchRole
{
	fn from(match_condition: Match<Id>) -> Self
	{
		Self { id: match_condition, ..Default::default() }
	}
}

impl From<MatchStr<String>> for MatchRole
{
	fn from(match_condition: MatchStr<String>) -> Self
	{
		Self { name: match_condition, ..Default::default() }
	}
}

impl From<Role> for MatchRole
{
	fn from(user: Role) -> Self
	{
		Self {
			id: user.id().into(),
			password_ttl: user.password_ttl().map(|d| Serde::from(d).into()).into(),
			name: user.name.into(),
		}
	}
}

impl From<String> for MatchRole
{
	fn from(s: String) -> Self
	{
		MatchStr::from(s).into()
	}
}
