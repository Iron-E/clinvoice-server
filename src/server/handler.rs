use core::marker::PhantomData;
use std::collections::HashSet;

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{
	extract::State,
	headers::{authorization::Basic, Authorization},
	http::StatusCode,
	routing::{self, MethodRouter},
	Extension,
	Json,
	TypedHeader,
};
use sqlx::{Database, Executor, Pool, Result, Transaction};
use tracing::Instrument;
use winvoice_adapter::{schema::LocationAdapter, Retrievable};
use winvoice_match::{Match, MatchDepartment, MatchEmployee, MatchExpense, MatchJob, MatchOption, MatchTimesheet};
use winvoice_schema::{chrono::Utc, Currency, Employee, Expense, Location, Timesheet};

use super::{
	auth::{AuthContext, DbUserStore, UserStore},
	response::{LoginResponse, LogoutResponse, Response, ResponseResult},
	ServerState,
};
use crate::{
	api::{
		request,
		response::{Get, Post},
		Code,
		Status,
	},
	permissions::{Action, Object},
	r#match::MatchUser,
	schema::{Adapter, User},
};

/// Map `result` of creating some enti`T`y into a [`ResponseResult`].
fn create<T>(result: Result<T>, on_success: Code) -> ResponseResult<Post<T>>
{
	result.map_or_else(
		|e| Err(Response::from(Post::from(Status::from(&e)))),
		|t| Ok(Response::from(Post::new(t.into(), on_success.into()))),
	)
}

/// [Retrieve](Retrievable::retrieve) using `R`, and map the result into a [`ResponseResult`].
async fn retrieve<R>(
	pool: &Pool<R::Db>,
	condition: R::Match,
	on_success: Code,
) -> ResponseResult<Get<<R as Retrievable>::Entity>>
where
	R: Retrievable,
{
	R::retrieve(pool, condition).await.map_or_else(
		|e| Err(Response::from(Get::from(Status::from(&e)))),
		|vec| Ok(Response::from(Get::new(vec, on_success.into()))),
	)
}

/// Return a [`ResponseResult`] for when a [`User`] tries to GET something, but they *effectively*
/// have no permissions (rather than outright having no permissions).
fn no_effective_get_perms<T>() -> ResponseResult<Get<T>>
{
	Ok(Response::from(Get::new(Default::default(), Code::SuccessForPermissions.into())))
}

const fn todo(msg: &'static str) -> (StatusCode, &'static str)
{
	(StatusCode::NOT_IMPLEMENTED, msg)
}

/// Create routes which are able to be implemented generically.
macro_rules! route {
	($Entity:ident) => {
		routing::delete(|| async move { todo("Delete method not implemented") })
			.get(
				|Extension(user): Extension<User>,
				 State(state): State<ServerState<A::Db>>,
				 Json(request): Json<request::Get<<A::$Entity as Retrievable>::Match>>| async move {
					state
						.enforce_permission::<Get<<A::$Entity as Retrievable>::Entity>>(
							&user,
							Object::$Entity,
							Action::Retrieve,
						)
						.await?;

					let condition = request.into_condition();
					A::$Entity::retrieve(state.pool(), condition).await.map_or_else(
						|e| Err(Response::from(Get::<<A::$Entity as Retrievable>::Entity>::from(Status::from(&e)))),
						|vec| Ok(Response::from(Get::new(vec, Code::Success.into()))),
					)
				},
			)
			.patch(|| async move { todo("Update method not implemented") })
	};
}

/// A handler for [`routes`](crate::api::routes).
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Handler<A>
{
	phantom: PhantomData<A>,
}

impl<A> Handler<A>
where
	A: Adapter,
	DbUserStore<A::Db>: UserStore,
	for<'connection> &'connection mut <A::Db as Database>::Connection: Executor<'connection, Database = A::Db>,
	for<'connection> &'connection mut Transaction<'connection, A::Db>: Executor<'connection, Database = A::Db>,
{
	/// The handler for the [`routes::CONTACT`](crate::api::routes::CONTACT).
	pub fn contact(&self) -> MethodRouter<ServerState<A::Db>>
	{
		route!(Contact).post(|| async move { todo("contact create") })
	}

	/// The handler for the [`routes::DEPARTMENT`](crate::api::routes::DEPARTMENT).
	pub fn department(&self) -> MethodRouter<ServerState<A::Db>>
	{
		routing::delete(|| async move { todo("Delete method not implemented") })
			.get(
				|Extension(user): Extension<User>,
				 State(state): State<ServerState<A::Db>>,
				 Json(request): Json<request::Get<MatchDepartment>>| async move {
					let mut condition = request.into_condition();
					let code = match state.department_permissions(&user, Action::Retrieve).await?
					{
						Object::Department => Code::Success,

						// HACK: no if-let guards…
						Object::AssignedDepartment if user.employee().is_some() =>
						{
							condition.id &= user.employee().unwrap().department.id.into();
							Code::SuccessForPermissions
						},

						// they have no department, so they *effectively* can't retrieve departments.
						Object::AssignedDepartment => return no_effective_get_perms(),
						p => p.unreachable(),
					};

					retrieve::<A::Department>(state.pool(), condition, code).await
				},
			)
			.patch(|| async move { todo("Update method not implemented") })
			.post(|| async move { todo("department create") })
	}

	/// The handler for the [`routes::EMPLOYEE`](crate::api::routes::EMPLOYEE).
	pub fn employee(&self) -> MethodRouter<ServerState<A::Db>>
	{
		routing::delete(|| async move { todo("Delete method not implemented") })
			.get(
				|Extension(user): Extension<User>,
				 State(state): State<ServerState<A::Db>>,
				 Json(request): Json<request::Get<MatchEmployee>>| async move {
					let mut condition = request.into_condition();
					let code = match state.employee_permissions::<Get<Employee>>(&user, Action::Retrieve).await?
					{
						Some(Object::Employee) => Code::Success,

						// HACK: no if-let guards…
						Some(Object::EmployeeInDepartment) if user.employee().is_some() =>
						{
							condition.department.id &= user.employee().unwrap().department.id.into();
							Code::SuccessForPermissions
						},

						// HACK: no if-let guards…
						None if user.employee().is_some() =>
						{
							condition.id &= user.employee().unwrap().id.into();
							Code::SuccessForPermissions
						},

						Some(Object::EmployeeInDepartment) | None => return no_effective_get_perms(),
						Some(p) => p.unreachable(),
					};

					retrieve::<A::Employee>(state.pool(), condition, code).await
				},
			)
			.patch(|| async move { todo("Update method not implemented") })
			.post(|| async move { todo("employee create") })
	}

	/// The handler for the [`routes::EXPENSE`](crate::api::routes::EXPENSE).
	pub fn expense(&self) -> MethodRouter<ServerState<A::Db>>
	{
		routing::delete(|| async move { todo("Delete method not implemented") })
			.get(
				|Extension(user): Extension<User>,
				 State(state): State<ServerState<A::Db>>,
				 Json(request): Json<request::Get<MatchExpense>>| async move {
					let permission = state.expense_permissions::<Get<Expense>>(&user, Action::Retrieve).await?;

					// The user has no department, and no employee record, so they effectively cannot retrieve
					// expenses.
					if permission != Object::Expenses && user.employee().is_none()
					{
						return no_effective_get_perms();
					}

					let condition = request.into_condition();

					let mut vec = A::Expenses::retrieve(state.pool(), condition)
						.await
						.map_err(|e| Response::from(Get::<Expense>::from(Status::from(&e))))?;

					let code = match permission
					{
						Object::Expenses => Code::Success,

						// The user can only get expenses iff they are in the same department, or were created
						// by that user.
						p =>
						{
							match user.employee()
							{
								Some(emp) =>
								{
									// retrieve IDs of expenses which the user has permission to access.
									// NOTE: `Timesheet::retrieve` retrieves *ALL* expenses for a timesheet, not just
									// the       ones which match the `expenses` field. Thus we still have to perform a
									// second       filter below.
									let matching = A::Timesheet::retrieve(state.pool(), MatchTimesheet {
										expenses: MatchExpense {
											id: Match::Or(vec.iter().map(|x| x.id.into()).collect()),
											..Default::default()
										}
										.into(),
										..match p
										{
											Object::ExpensesInDepartment =>
											{
												MatchJob::from(MatchDepartment::from(emp.department.id)).into()
											},
											Object::CreatedExpenses => MatchEmployee::from(emp.id).into(),
											_ => p.unreachable(),
										}
									})
									.await
									.map_or_else(
										|e| Err(Response::from(Get::from(Status::from(&e)))),
										|vec| {
											Ok(vec
												.into_iter()
												.flat_map(|t| t.expenses.into_iter().map(|x| x.id))
												.collect::<HashSet<_>>())
										},
									)?;

									vec.retain(|x| matching.contains(&x.id));
								},

								None => unreachable!("Should have been returned earlier for {permission:?}"),
							};

							Code::SuccessForPermissions
						},
					};

					Ok::<_, Response<_>>(Response::from(Get::new(vec, code.into())))
				},
			)
			.patch(|| async move { todo("Update method not implemented") })
			.post(|| async move { todo("expense create") })
	}

	/// The handler for the [`routes::JOB`](crate::api::routes::JOB).
	pub fn job(&self) -> MethodRouter<ServerState<A::Db>>
	{
		routing::delete(|| async move { todo("Delete method not implemented") })
			.get(
				|Extension(user): Extension<User>,
				 State(state): State<ServerState<A::Db>>,
				 Json(request): Json<request::Get<MatchJob>>| async move {
					let mut condition = request.into_condition();

					let code = match state.job_permissions(&user, Action::Retrieve).await?
					{
						Object::Job => Code::Success,

						// HACK: no if-let guards…
						Object::JobInDepartment if user.employee().is_some() =>
						{
							condition.departments &=
								MatchDepartment::from(user.employee().unwrap().department.id).into();
							Code::SuccessForPermissions
						},

						Object::JobInDepartment => return no_effective_get_perms(),
						p => p.unreachable(),
					};

					retrieve::<A::Job>(state.pool(), condition, code).await
				},
			)
			.patch(|| async move { todo("Update method not implemented") })
			.post(|| async move { todo("job create") })
	}

	/// The handler for the [`routes::LOCATION`](crate::api::routes::LOCATION).
	pub fn location(&self) -> MethodRouter<ServerState<A::Db>>
	{
		route!(Location).post(
			|Extension(user): Extension<User>,
			 State(state): State<ServerState<A::Db>>,
			 Json(request): Json<request::Post<(Option<Currency>, String, Option<Location>)>>| async move {
				state.enforce_permission(&user, Object::Location, Action::Create).await?;
				let (currency, name, outer) = request.into_args();
				create(A::Location::create(state.pool(), currency, name, outer).await, Code::Success)
			},
		)
	}

	/// The handler for the [`routes::LOGIN`](crate::api::routes::LOGIN).
	pub fn login(&self) -> MethodRouter<ServerState<A::Db>>
	{
		routing::get(
			|mut auth: AuthContext<A::Db>,
			 State(state): State<ServerState<A::Db>>,
			 TypedHeader(credentials): TypedHeader<Authorization<Basic>>| {
				async move {
					let user = match A::User::retrieve(state.pool(), MatchUser {
						username: credentials.username().to_owned().into(),
						..Default::default()
					})
					.await
					.map(|mut v| v.pop())
					{
						Ok(Some(u)) => u,
						Ok(None) => return Err(LoginResponse::invalid_credentials(None)),
						Err(e) => return Err(LoginResponse::from(e)),
					};

					PasswordHash::new(user.password()).map_or_else(
						|e| {
							tracing::error!(
								"Failed to decode user {}'s password hash stored in database",
								user.username()
							);
							Err(LoginResponse::new(
								StatusCode::INTERNAL_SERVER_ERROR,
								Status::new(Code::EncodingError, e.to_string()),
							))
						},
						|hash| {
							Argon2::default().verify_password(credentials.password().as_bytes(), &hash).map_err(|e| {
								tracing::info!("Invalid login attempt for user {}", user.username());
								LoginResponse::from(e)
							})
						},
					)?;

					// HACK: no if-let chain…
					if let Some(date) = user.password_expires()
					{
						if date < Utc::now()
						{
							tracing::info!("User {} attempted to login with expired password", user.username());
							return Err(LoginResponse::expired(date));
						}
					}

					auth.login(&user).await.map_or_else(
						|e| {
							const CODE: Code = Code::LoginError;
							tracing::error!("Failed to to log in user {}: {e}", user.username());
							Err(LoginResponse::new(CODE.into(), Status::new(CODE, e.to_string())))
						},
						|_| Ok(LoginResponse::from(Code::Success)),
					)
				}
				.instrument(tracing::info_span!("login_handler"))
			},
		)
	}

	/// The handler for the [`routes::LOGOUT`](crate::api::routes::LOGOUT).
	pub fn logout(&self) -> MethodRouter<ServerState<A::Db>>
	{
		routing::get(|mut auth: AuthContext<A::Db>| {
			async move {
				auth.logout().await;
				LogoutResponse::from(Code::Success)
			}
			.instrument(tracing::info_span!("logout_handler"))
		})
	}

	/// Create a new [`Handler`].
	pub fn new() -> Self
	{
		Self { phantom: PhantomData }
	}

	/// The handler for the [`routes::ORGANIZATION`](crate::api::routes::ORGANIZATION).
	pub fn organization(&self) -> MethodRouter<ServerState<A::Db>>
	{
		route!(Organization).post(|| async move { todo("organization create") })
	}

	/// The handler for the [`routes::ROLE`](crate::api::routes::ROLE).
	pub fn role(&self) -> MethodRouter<ServerState<A::Db>>
	{
		route!(Role).post(|| async move { todo("role create") })
	}

	/// The handler for the [`routes::TIMESHEET`](crate::api::routes::TIMESHEET).
	pub fn timesheet(&self) -> MethodRouter<ServerState<A::Db>>
	{
		routing::delete(|| async move { todo("Delete method not implemented") })
			.get(
				|Extension(user): Extension<User>,
				 State(state): State<ServerState<A::Db>>,
				 Json(request): Json<request::Get<MatchTimesheet>>| async move {
					let mut condition = request.into_condition();
					let code = match state.timesheet_permissions::<Get<Timesheet>>(&user, Action::Retrieve).await?
					{
						Object::Timesheet => Code::Success,

						// HACK: no if-let guards
						Object::TimesheetInDepartment if user.employee().is_some() =>
						{
							condition.job.departments &=
								MatchDepartment::from(user.employee().unwrap().department.id).into();
							Code::SuccessForPermissions
						},

						// HACK: no if-let guards
						Object::CreatedTimesheet if user.employee().is_some() =>
						{
							condition.employee.id &= user.employee().unwrap().id.into();
							Code::SuccessForPermissions
						},

						Object::TimesheetInDepartment | Object::CreatedTimesheet => return no_effective_get_perms(),
						p => p.unreachable(),
					};

					retrieve::<A::Timesheet>(state.pool(), condition, code).await
				},
			)
			.patch(|| async move { todo("Update method not implemented") })
			.post(|| async move { todo("timesheet create") })
	}

	/// The handler for the [`routes::USER`](crate::api::routes::USER).
	pub fn user(&self) -> MethodRouter<ServerState<A::Db>>
	{
		routing::delete(|| async move { todo("Delete method not implemented") })
			.get(
				|Extension(user): Extension<User>,
				 State(state): State<ServerState<A::Db>>,
				 Json(request): Json<request::Get<MatchUser>>| async move {
					let mut condition = request.into_condition();
					let code = match state.user_permissions::<Get<User>>(&user, Action::Retrieve).await?
					{
						Some(Object::User) => Code::Success,

						// HACK: no if-let guards
						Some(Object::UserInDepartment) if user.employee().is_some() =>
						{
							let dpt_id = user.employee().unwrap().department.id;
							condition.employee = match condition.employee
							{
								MatchOption::Any => Some(MatchDepartment::from(dpt_id).into()).into(),
								e => e.map(|mut m| {
									m.department.id &= dpt_id.into();
									m
								}),
							};

							Code::SuccessForPermissions
						},

						// HACK: no if-let guards
						None if user.employee().is_some() =>
						{
							let emp_id = user.employee().unwrap().id;
							condition.employee = match condition.employee
							{
								MatchOption::Any => Some(emp_id.into()).into(),
								e => e.map(|mut m| {
									m.id &= emp_id.into();
									m
								}),
							};

							Code::SuccessForPermissions
						},

						Some(Object::UserInDepartment) | None => return no_effective_get_perms(),
						Some(p) => p.unreachable(),
					};

					retrieve::<A::User>(state.pool(), condition, code).await
				},
			)
			.patch(|| async move { todo("Update method not implemented") })
			.post(|| async move { todo("user create") })
	}
}