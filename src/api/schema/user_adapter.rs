//! Contains a version of e.g. [`TimesheetAdapter`](winvoice_adapter::schema::TimesheetAdapter) for
//! [`User`](super::User).

use sqlx::{Executor, Result};
use winvoice_adapter::{Deletable, Retrievable, Updatable};
use winvoice_schema::{
	chrono::{DateTime, Utc},
	Employee,
};

use super::User;
use crate::api::r#match::MatchUser;

/// Implementors of this trait may act as an [adapter](super) for [`Employee`]s.
#[async_trait::async_trait]
pub trait UserAdapter:
	Deletable<Entity = User>
	+ Retrievable<
		Db = <Self as Deletable>::Db,
		Entity = <Self as Deletable>::Entity,
		Match = MatchUser,
	> + Updatable<Db = <Self as Deletable>::Db, Entity = <Self as Deletable>::Entity>
{
	/// Initialize and return a new [`Employee`] via the `connection`.
	async fn create<'connection, Conn>(
		connection: Conn,
		employee: Option<Employee>,
		role: String,
		password: String,
		password_expires: Option<DateTime<Utc>>,
		username: String,
	) -> Result<<Self as Deletable>::Entity>
	where
		Conn: Executor<'connection, Database = <Self as Deletable>::Db>;
}

#[cfg(feature = "postgres")]
mod postgres
{
	// TODO: impl UserAdapter for PgUser
}
