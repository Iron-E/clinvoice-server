//! Contains [`From`] implementations for [`MatchRole`].

use super::{Id, Match, MatchRole};

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
