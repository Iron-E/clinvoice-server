//! Contains [`From`] implementations for [`MatchUser`].

use super::{Id, Match, MatchUser};

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
