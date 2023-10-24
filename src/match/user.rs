//! Contains a [`Match`](winvoice_match::Match) type for [`User`](crate::schema::User)s

mod from;

use serde::{Deserialize, Serialize};
use winvoice_match::{Match, MatchEmployee, MatchOption, MatchStr};
use winvoice_schema::{
	chrono::{DateTime, Utc},
	Id,
};

use super::MatchRole;

/// A [`User`](crate::schema::User) with [matchable](winvoice_match) fields.
///
/// [`MatchUser`] matches IFF all of its fields also match.
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
/// # use pretty_assertions::assert_eq;
/// # use winvoice_match::{MatchEmployee, MatchStr};
/// # use winvoice_schema::chrono::{NaiveDate, Utc};
/// # use winvoice_server::r#match::{MatchRole, MatchUser};
/// // JSON
/// # assert_eq!(serde_json::from_str::<MatchUser>(r#"
/// {
///   "employee": {"some": {
///     "name": {"regex": "[Aa]ndy$"}
///   }},
///   "password": "asdlkjasfhjdklasdklj",
///   "password_set": "2070-01-01T00:00:00Z",
///   "role": {"name": "Admin"},
///   "username": "admin"
/// }
/// # "#).unwrap(), MatchUser {
/// #   employee: Some(MatchEmployee {
/// #     name: MatchStr::Regex("[Aa]ndy$".into()),
/// #     ..Default::default()
/// #   }).into(),
/// #   password: "asdlkjasfhjdklasdklj".to_owned().into(),
/// #   password_set: NaiveDate::from_ymd_opt(2070, 1, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap().and_utc(),
/// #   role: MatchRole { name: "Admin".to_owned().into(), ..Default::default() },
/// #   username: "admin".to_owned().into(),
/// #   ..Default::default()
/// # });
/// ```
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct MatchUser
{
	#[allow(missing_docs)]
	#[serde(default)]
	pub employee: MatchOption<MatchEmployee>,

	#[allow(missing_docs)]
	#[serde(default)]
	pub id: Match<Id>,

	#[allow(missing_docs)]
	#[serde(default)]
	pub password: MatchStr<String>,

	#[allow(missing_docs)]
	#[serde(default)]
	pub password_set: Match<DateTime<Utc>>,

	#[allow(missing_docs)]
	#[serde(default)]
	pub role: MatchRole,

	#[allow(missing_docs)]
	#[serde(default)]
	pub username: MatchStr<String>,
}
