//! Contains code to help synchronize data access in async contexts.

use std::sync::Arc;

use tokio::sync::RwLock;

/// A wrapper for `T` which allows synchronizes data access.
pub type Lock<T> = Arc<RwLock<T>>;

/// Create new [`Sync`]hronous data.
#[allow(clippy::arc_with_non_send_sync)]
pub fn new<T>(t: T) -> Lock<T>
{
	Arc::new(RwLock::new(t))
}
