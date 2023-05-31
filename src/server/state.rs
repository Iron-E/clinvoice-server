//! Contains data and functions for the [`State`] which is shared by the [`Server`](super::Server).

mod clone;

use casbin::{CoreApi, Enforcer};
use sqlx::{Database, Pool};
use winvoice_adapter::Retrievable;

use crate::{
	api::{
		r#match::MatchRole,
		schema::{Role, User},
	},
	lock::Lock,
	permissions::{Action, Object},
	DynResult,
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
	pub async fn has_permission<R>(
		&self,
		user: User,
		object: Object,
		action: Action,
	) -> DynResult<bool>
	where
		R: Retrievable<Db = Db, Entity = Role, Match = MatchRole>,
	{
		let permissions = self.permissions.read().await;
		if permissions.enforce((user.username(), object, action))?
		{
			return Ok(true);
		}

		let role = R::retrieve(&self.pool, user.role_id().into())
			.await
			.and_then(|mut v| v.pop().ok_or(sqlx::Error::RowNotFound))?;

		permissions.enforce((role.name(), object, action)).map_err(|e| e.into())
	}

	/// Get the [`Pool`] of connections to the [`Database`].
	pub const fn pool(&self) -> &Pool<Db>
	{
		&self.pool
	}
}
