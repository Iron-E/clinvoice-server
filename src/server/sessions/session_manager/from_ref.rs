//! Contains the [`SessionManager`] implementation of [`FromRef`] such that
//! [`PrivateCookieJar`](axum_extra::extract::cookie::PrivateCookieJar)s can access the
//! [`Refresh`](super::Refresh) secret.

use axum::extract::FromRef;
use sqlx::Database;

use super::{Key, SessionManager};

// this impl tells `SignedCookieJar` how to access the key from our state
impl<Db> FromRef<SessionManager<Db>> for Key
where
	Db: Database,
{
	fn from_ref(state: &SessionManager<Db>) -> Self
	{
		state.refresh_secret.clone()
	}
}
