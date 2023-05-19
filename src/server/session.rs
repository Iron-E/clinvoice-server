//! Manages sessions and logging in.

mod login;

use std::collections::HashMap;

pub use login::Login;
use once_cell::sync::Lazy;
use uuid::Uuid;
use winvoice_schema::chrono::{DateTime, Local};

static SESSIONS: Lazy<HashMap<Uuid, Session>> = Lazy::new(HashMap::new);

/// Represents a user who has successfully logged in, and may *stay* logged.
struct Session
{
	date: DateTime<Local>,
	username: String,
	password: String,
}
