//! Contains data and functions for the [`State`] which is shared by the [`Server`](super::Server).

mod clone;

use casbin::{CoreApi, Enforcer};
use sqlx::{Database, Pool};

use crate::{
	api::{Code, Status},
	lock::Lock,
	permissions::{Action, Object},
	schema::User,
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
		user: &User,
		object: Object,
		action: Action,
	) -> Result<(), Status>
	{
		let permissions = self.permissions.read().await;

		match permissions
			.enforce((user.role().name(), object, action))
			.and_then(|role_authorized| {
				Ok(role_authorized || permissions.enforce((user.username(), object, action))?)
			})
			.map_err(|e| Status::from(&e))?
		{
			true => Ok(()),
			false => Err(Status::new(
				Code::Unauthorized,
				format!("{} is not authorized to {action} {object}s", user.username()),
			)),
		}
	}

	/// Get the [`Pool`] of connections to the [`Database`].
	pub const fn pool(&self) -> &Pool<Db>
	{
		&self.pool
	}
}
