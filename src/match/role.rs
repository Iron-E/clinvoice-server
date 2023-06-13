//! Contains a [`Match`](winvoice_match::Match) type for [`User`](crate::schema::User)s

mod from;

use core::time::Duration;

use humantime_serde::Serde;
use serde::{Deserialize, Serialize};
use winvoice_match::{Match, MatchOption, MatchStr};
use winvoice_schema::Id;

/// A [`Timesheet`](winvoice_schema::Timesheet) with [matchable](winvoice_match) fields.
///
/// [`MatchTimesheet`] matches IFF all of its fields also match.
///
/// # Examples
///
/// Requires the `serde` feature. If any field is omitted, it will be set to the
/// [`Default`] for its type.
///
/// See the documentation for the type of each top-level field (e.g. `id`, `employee`) for
/// information about the types of matching operations which each field supports.
///
/// ```rust
/// # use core::time::Duration;
/// # use pretty_assertions::assert_eq;
/// # use winvoice_match::*;
/// # use winvoice_server::r#match::MatchRole;
/// // JSON
/// # assert_eq!(serde_json::from_str::<MatchRole>(r#"
/// {
///   "id": "any",
///   "name": {"contains": "Peter"},
///   "password_ttl": {"some": {"less_than": "1d"}}
/// }
/// # "#).unwrap(), MatchRole {
/// #   name: MatchStr::Contains("Peter".into()),
/// #   password_ttl: Some(Match::LessThan(Duration::from_secs(60 * 60 * 24).into())).into(),
/// #   ..Default::default()
/// # });
/// ```
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct MatchRole
{
	#[allow(missing_docs)]
	#[serde(default)]
	pub id: Match<Id>,

	#[allow(missing_docs)]
	#[serde(default)]
	pub name: MatchStr<String>,

	#[allow(missing_docs)]
	#[serde(default)]
	pub password_ttl: MatchOption<Match<Serde<Duration>>>,
}
