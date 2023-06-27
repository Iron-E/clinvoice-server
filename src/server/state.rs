//! Contains data and functions for the [`State`] which is shared by the [`Server`](super::Server).

mod clone;

use casbin::{CoreApi, Enforcer};
use sqlx::{Database, Pool};

use super::Response;
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

	/// Check [`has_permission`](Self::has_permission), but also return [`Err`] if the [`Result`]
	/// was [`Ok(false)`].
	pub async fn enforce_permission<R>(
		&self,
		user: &User,
		object: Object,
		action: Action,
	) -> Result<(), Response<R>>
	where
		R: AsRef<Code> + From<Status>,
	{
		self.has_permission(user, object, action).await.and_then(|has_permission| {
			match has_permission
			{
				true => Ok(()),
				false => Err(Response::from(Status::from((user, object, action)).into())),
			}
		})
	}

	/// Check whether `subject` has permission to perform an `action` on the `object`.
	pub async fn has_permission<R>(
		&self,
		user: &User,
		object: Object,
		action: Action,
	) -> Result<bool, Response<R>>
	where
		R: AsRef<Code> + From<Status>,
	{
		let permissions = self.permissions.read().await;
		permissions
			.enforce((user.role().name(), object, action))
			.and_then(|role_authorized| {
				Ok(role_authorized || permissions.enforce((user.username(), object, action))?)
			})
			.map_err(|e| Response::from(Status::from(&e).into()))
	}

	/// Get the [`Pool`] of connections to the [`Database`].
	pub const fn pool(&self) -> &Pool<Db>
	{
		&self.pool
	}
}
