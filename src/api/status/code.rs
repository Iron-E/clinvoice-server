//! This module contains data used in reporting the specific code which resulted from an action.

mod display;

use serde::{Deserialize, Serialize};

/// The specific outcome of an operation.
#[derive(
	Clone, Copy, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize,
)]
pub enum Code
{
	/// There was an attempt to log in, but it failed because the credentials provided were not
	/// accepted by the database.
	InvalidCredentials = 2,

	/// A user has successfully logged in.
	LoggedIn = 1,

	/// A user has successfully logged out.
	LoggedOut = 4,

	/// An uncategorized type of action was taken.
	#[default]
	Other = 0,

	/// A user has attempted to perform an operation on the database while not having the correct
	/// permissions.
	Unauthorized = 3,
}
