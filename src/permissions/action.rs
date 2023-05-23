//! Contains all the actions which may be taken by users.

use serde::{Deserialize, Serialize};

/// The actions which users may have permission to take.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Action
{
	Create,
	Delete,
	Retrieve,
	Update,
}
