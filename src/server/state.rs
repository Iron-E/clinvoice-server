//! Contains data and functions for the [`State`] which is shared by the [`Server`](super::Server).

use casbin::{CoreApi, EnforceArgs, Enforcer};
use sqlx::{Database, Pool};

use crate::permissions::{Action, Object};

/// The state which is shared by the server.
pub struct State<Db>
where
	Db: Database,
{
	/// The user permissions.
	permissions: Enforcer,

	/// The [`Pool`] of connections to the [`Database`].
	pool: Pool<Db>,
}

impl<Db> State<Db>
where
	Db: Database,
{
	/// Create new [`State`]
	pub const fn new(permissions: Enforcer, pool: Pool<Db>) -> Self
	{
		Self { pool, permissions }
	}

	/// Check whether `subject` has permission to `action` on `object`.
	pub fn has_permission<TSubject>(
		&self,
		subject: TSubject,
		object: Object,
		action: Action,
	) -> casbin::Result<bool>
	where
		(TSubject, Object, Action): EnforceArgs,
	{
		self.permissions.enforce((subject, object, action))
	}

	/// Get the [`Pool`] of connections to the [`Database`].
	pub const fn pool(&self) -> &Pool<Db>
	{
		&self.pool
	}
}
