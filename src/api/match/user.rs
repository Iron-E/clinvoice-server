//! Contains a [`Match`](winvoice_match::Match) type for [`User`](crate::api::schema::User)s

mod from;

use serde::{Deserialize, Serialize};
use winvoice_match::{Match, MatchOption, MatchStr};
use winvoice_schema::{chrono::NaiveDateTime, Id};

/// A [`Timesheet`](winvoice_schema::Timesheet) with [matchable](winvoice_match) fields.
///
/// [`MatchTimesheet`] matches IFF all of its fields also match.
///
/// # Examples
///
/// ## YAML
///
/// Requires the `serde` feature. If any field is omitted, it will be set to the
/// [`Default`] for its type.
///
/// See the documentation for the type of each top-level field (e.g. `id`, `employee`) for
/// information about the types of matching operations which each field supports.
///
/// ```rust
/// # assert!(serde_yaml::from_str::<winvoice_match::MatchTimesheet>(r#"
/// id: any
/// employee:
///   name:
///     regex: '^[JR]on$'
/// expenses:
///   contains:
///     category:
///       equal_to: "Travel"
/// job:
///   client:
///     name:
///       contains: "Interational"
/// time_begin:
///   less_than: "2022-01-01T00:00:00"
/// time_end: none
/// work_notes: any
/// # "#).is_ok());
/// ```
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct MatchUser
{
	#[allow(missing_docs)]
	#[serde(default)]
	pub id: Match<Id>,
}
