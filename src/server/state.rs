//! Contains data and functions for the [`State`] which is shared by the [`Server`](super::Server).

mod clone;

use casbin::{CoreApi, EnforceArgs, Enforcer};
use sqlx::{Database, Pool};

use crate::{
	lock::Lock,
	permissions::{Action, Object},
};

/// The state which is shared by the server.
pub struct State<Db>
where
	Db: Database,
{
	/// The user permissions.
	permissions: Lock<Enforcer>,

	/// The [`Pool`] of connections to the [`Database`].
	pool: Pool<Db>,
}

impl<Db> State<Db>
where
	Db: Database,
{
	/// Create new [`State`]
	pub const fn new(permissions: Lock<Enforcer>, pool: Pool<Db>) -> Self
	{
		Self { pool, permissions }
	}

	/// Check whether `subject` has permission to `action` on `object`.
	pub async fn has_permission<TSubject>(
		&self,
		subject: TSubject,
		object: Object,
		action: Action,
	) -> casbin::Result<bool>
	where
		(TSubject, Object, Action): EnforceArgs,
	{
		self.permissions.read().await.enforce((subject, object, action))
	}

	/// Get the [`Pool`] of connections to the [`Database`].
	pub const fn pool(&self) -> &Pool<Db>
	{
		&self.pool
	}
}
