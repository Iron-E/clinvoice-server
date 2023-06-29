//! Contains data and functions for the [`State`] which is shared by the [`Server`](super::Server).

mod clone;

use casbin::{CoreApi, Enforcer};
use sqlx::{Database, Pool};

use super::Response;
use crate::{
	api::{Code, Status},
	bool_ext::BoolExt,
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
	/// Check whether `user` [`has_permission`](Self::has_permission) to perform an `action` on
	/// [`Object::Department`].
	///
	/// If the `user` does not have this permission, it will [`enforce`](Self::enforce_permission)
	/// the [`Object::AssignedDepartment`] permission.
	pub async fn department_permissions<R>(
		&self,
		user: &User,
		action: Action,
	) -> Result<Object, Response<R>>
	where
		R: AsRef<Code> + From<Status>,
	{
		let mut object = Object::Department;
		let can_get_all = self.has_permission::<R>(&user, object, action).await?;

		if !can_get_all
		{
			// if they cannot get their assigned department, then they cannot
			// retrieve ANY departments.
			object = Object::AssignedDepartment;
			self.enforce_permission::<R>(&user, object, action).await?;
		}

		Ok(object)
	}

	/// First, check whether `user` [`has_permission`](Self::has_permission) to perform an `action`
	/// on [`Object::Employee`].
	///
	/// If that permission is missing, then check whether `user`
	/// [`has_permission`](Self::has_permission) to perform an `action` on
	/// [`Object::EmployeeInDepartment`].
	///
	/// If no relevant permissions were found, [`None`] is returned. This indicates that the `user`
	/// can only operate on their own employee record.
	pub async fn employee_permissions<R>(
		&self,
		user: &User,
		action: Action,
	) -> Result<Option<Object>, Response<R>>
	where
		R: AsRef<Code> + From<Status>,
	{
		let mut object = Object::Employee;
		let can_get_all = self.has_permission::<R>(&user, object, action).await?;

		let can_get_in_dept = can_get_all || {
			object = Object::EmployeeInDepartment;
			self.has_permission::<R>(&user, object, action).await?
		};

		Ok(can_get_in_dept.then_or(None, || object.into()))
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
			has_permission.then_some_or_else(
				|| Err(Response::from(Status::from((user, object, action)).into())),
				Ok(()),
			)
		})
	}

	/// First, check whether `user` [`has_permission`](Self::has_permission) to perform an `action`
	/// on [`Object::Expenses`].
	///
	/// If that permission is missing, then check whether `user`
	/// [`has_permission`](Self::has_permission) to perform an `action` on
	/// [`Object::ExpensesInDepartment`].
	///
	/// If *that* permission is missing, then [`enforce`](Self::enforce_permission) the
	/// [`Object::CreatedExpenses`] permission.
	pub async fn expense_permissions<R>(
		&self,
		user: &User,
		action: Action,
	) -> Result<Object, Response<R>>
	where
		R: AsRef<Code> + From<Status>,
	{
		let mut object = Object::Expenses;
		let can_get_all = self.has_permission::<R>(&user, object, action).await?;

		let can_get_in_dept = can_get_all || {
			object = Object::ExpensesInDepartment;
			self.has_permission::<R>(&user, object, action).await?
		};

		if !can_get_in_dept
		{
			object = Object::CreatedExpenses;
			self.enforce_permission::<R>(&user, object, action).await?;
		}

		Ok(object)
	}

	/// Check whether `user` has permission to perform an `action` on the `object`.
	async fn has_permission<R>(
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

	/// First, check whether `user` [`has_permission`](Self::has_permission) to perform an `action`
	/// on [`Object::Job`].
	///
	/// If that permission is missing, then [`enforce`](Self::enforce_permission) that `user` can
	/// perform an `action` on [`Object::JobInDepartment`].
	pub async fn job_permissions<R>(
		&self,
		user: &User,
		action: Action,
	) -> Result<Object, Response<R>>
	where
		R: AsRef<Code> + From<Status>,
	{
		let mut object = Object::Job;
		let can_get_all = self.has_permission::<R>(&user, object, action).await?;

		if !can_get_all
		{
			object = Object::JobInDepartment;
			self.has_permission::<R>(&user, object, action).await?;
		}

		Ok(object)
	}

	/// Create new [`State`]
	pub const fn new(permissions: Lock<Enforcer>, pool: Pool<Db>) -> Self
	{
		Self { pool, permissions }
	}

	/// Get the [`Pool`] of connections to the [`Database`].
	pub const fn pool(&self) -> &Pool<Db>
	{
		&self.pool
	}

	/// First, check whether `user` [`has_permission`](Self::has_permission) to perform an `action`
	/// on [`Object::Timesheet`].
	///
	/// If that permission is missing, then check whether `user`
	/// [`has_permission`](Self::has_permission) to perform an `action` on
	/// [`Object::TimesheetInDepartment`].
	///
	/// If *that* permission is missing, then [`enforce`](Self::enforce_permission) the
	/// [`Object::CreatedTimesheet`] permission.
	pub async fn timesheet_permissions<R>(
		&self,
		user: &User,
		action: Action,
	) -> Result<Object, Response<R>>
	where
		R: AsRef<Code> + From<Status>,
	{
		let mut object = Object::Timesheet;
		let can_get_all = self.has_permission::<R>(&user, object, action).await?;

		let can_get_in_dept = can_get_all || {
			object = Object::TimesheetInDepartment;
			self.has_permission::<R>(&user, object, action).await?
		};

		if !can_get_in_dept
		{
			object = Object::CreatedTimesheet;
			self.enforce_permission::<R>(&user, object, action).await?;
		}

		Ok(object)
	}

	/// First, check whether `user` [`has_permission`](Self::has_permission) to perform an `action`
	/// on [`Object::User`].
	///
	/// If that permission is missing, then check whether `user`
	/// [`has_permission`](Self::has_permission) to perform an `action` on
	/// [`Object::UserInDepartment`].
	///
	/// If no relevant permissions were found, [`None`] is returned. This indicates that the `user`
	/// can only operate on their own employee record.
	pub async fn user_permissions<R>(
		&self,
		user: &User,
		action: Action,
	) -> Result<Option<Object>, Response<R>>
	where
		R: AsRef<Code> + From<Status>,
	{
		let mut object = Object::User;
		let can_get_all = self.has_permission::<R>(&user, object, action).await?;

		let can_get_in_dept = can_get_all || {
			object = Object::UserInDepartment;
			self.has_permission::<R>(&user, object, action).await?
		};

		Ok(can_get_in_dept.then_or(None, || object.into()))
	}
}
