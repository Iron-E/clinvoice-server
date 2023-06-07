//! Contains data and functions for the [`State`] which is shared by the [`Server`](super::Server).

mod clone;

use casbin::{CoreApi, Enforcer};
use sqlx::{Database, Pool};

use crate::{
	api::schema::User,
	lock::Lock,
	permissions::{Action, Object},
};

/// The state which is shared by the server.
pub struct ServerState<Db>
where
	Db: Database,
{
	/// The user permissions.
	permissions: Lock<Enforcer>,

	/// The [`Pool`] of connections to the [`Database`].
	pool: Pool<Db>,
}

impl<Db> ServerState<Db>
where
	Db: Database,
{
	/// Create new [`State`]
	pub const fn new(permissions: Lock<Enforcer>, pool: Pool<Db>) -> Self
	{
		Self { pool, permissions }
	}

	/// Check whether `subject` has permission to `action` on `object`.
	pub async fn has_permission(
		&self,
		user: User,
		object: Object,
		action: Action,
	) -> casbin::Result<bool>
	{
		let permissions = self.permissions.read().await;
		let user_has_perms = permissions.enforce((user.username(), object, action))?;
		let role_has_perms =
			user_has_perms || permissions.enforce((user.role().name(), object, action))?;

		Ok(role_has_perms)
	}

	/// Get the [`Pool`] of connections to the [`Database`].
	pub const fn pool(&self) -> &Pool<Db>
	{
		&self.pool
	}
}
