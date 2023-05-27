//! Contains [`From`] implementations for [`MatchUser`].

use super::{Id, Match, MatchUser};

impl From<Id> for MatchTimesheet
{
	fn from(id: Id) -> Self
	{
		Match::from(id).into()
	}
}

impl From<Match<Id>> for MatchTimesheet
{
	fn from(match_condition: Match<Id>) -> Self
	{
		Self { id: match_condition, ..Default::default() }
	}
}
