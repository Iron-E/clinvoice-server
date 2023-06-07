//! Contains all the actions which may be taken by users.

mod display;

use serde::{Deserialize, Serialize};

/// The actions which users may have permission to take.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Action
{
	/// Permission to create [`Object`](super::Object)s.
	Create,

	/// Permission to delete [`Object`](super::Object)s.
	Delete,

	/// Permission to retrieve [`Object`](super::Object)s.
	Retrieve,

	/// Permission to update [`Object`](super::Object)s.
	Update,
}
