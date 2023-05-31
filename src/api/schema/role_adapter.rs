//! Contains a version of e.g. [`TimesheetAdapter`](winvoice_adapter::schema::TimesheetAdapter) for
//! [`User`](super::User).

use core::time::Duration;

use sqlx::{Executor, Result};
use winvoice_adapter::{Deletable, Retrievable, Updatable};

use super::Role;
use crate::api::r#match::MatchRole;

/// Implementors of this trait may act as an [adapter](super) for [`Employee`]s.
#[async_trait::async_trait]
pub trait RoleAdapter:
	Deletable<Entity = Role>
	+ Retrievable<
		Db = <Self as Deletable>::Db,
		Entity = <Self as Deletable>::Entity,
		Match = MatchRole,
	> + Updatable<Db = <Self as Deletable>::Db, Entity = <Self as Deletable>::Entity>
{
	/// Initialize and return a new [`Employee`] via the `connection`.
	async fn create<'connection, Conn>(
		connection: Conn,
		name: String,
		password_ttl: Option<Duration>,
	) -> Result<<Self as Deletable>::Entity>
	where
		Conn: Executor<'connection, Database = <Self as Deletable>::Db>;
}
