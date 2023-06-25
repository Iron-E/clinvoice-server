//! Contains [`From`] implementations for [`MatchRole`].

use super::{Id, Match, MatchRole, MatchStr};

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

impl From<String> for MatchRole
{
	fn from(s: String) -> Self
	{
		MatchStr::from(s).into()
	}
}
