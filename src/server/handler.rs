mod reason;

use core::{marker::PhantomData, time::Duration};
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
use reason::Reason;
use sqlx::{Database, Executor, Pool, Result};
use tracing::Instrument;
use winvoice_adapter::{
	schema::{
		ContactAdapter,
		DepartmentAdapter,
		EmployeeAdapter,
		ExpensesAdapter,
		LocationAdapter,
		OrganizationAdapter,
	},
	Deletable,
	Retrievable,
	Updatable,
};
use winvoice_match::{Match, MatchDepartment, MatchEmployee, MatchExpense, MatchJob, MatchOption, MatchTimesheet};
use winvoice_schema::{chrono::Utc, ContactKind, Currency, Department, Employee, Expense, Id, Location, Money};

use super::{
	auth::{AuthContext, DbUserStore, UserStore},
	response::{DeleteResponse, LoginResponse, LogoutResponse, PatchResponse, Response, ResponseResult},
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
	schema::{Adapter, RoleAdapter, User},
	twin_result::TwinResult,
	ResultExt,
};

/// Map `result` of creating some enti`T`y into a [`ResponseResult`].
fn create<T>(result: Result<T>, on_success: Code) -> ResponseResult<Post<T>>
{
	result.map_all(
		|t| Response::from(Post::new(t.into(), on_success.into())),
		|e| Response::from(Post::from(Status::from(&e))),
	)
}

/// [Retrieve](Retrievable::retrieve) using `R`, and map the result into a [`ResponseResult`].
async fn delete<D>(pool: &Pool<D::Db>, entities: Vec<D::Entity>, on_success: Code) -> TwinResult<DeleteResponse>
where
	D: Deletable,
	D::Entity: Sync,
	for<'con> &'con mut <D::Db as Database>::Connection: Executor<'con, Database = D::Db>,
{
	D::delete(pool, entities.iter()).await.map_all(|_| DeleteResponse::from(on_success), DeleteResponse::from)
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
	R::retrieve(pool, condition).await.map_all(
		|vec| Response::from(Get::new(vec, on_success.into())),
		|e| Response::from(Get::from(Status::from(&e))),
	)
}

/// [Retrieve](Retrievable::retrieve) using `R`, and map the result into a [`ResponseResult`].
async fn update<U>(pool: &Pool<U::Db>, entities: Vec<U::Entity>, on_success: Code) -> TwinResult<PatchResponse>
where
	U: Updatable,
	U::Entity: Sync,
{
	let mut tx = pool.begin().await.map_err(PatchResponse::from)?;
	U::update(&mut tx, entities.iter()).await.map_err(PatchResponse::from)?;
	tx.commit().await.map_all(|_| PatchResponse::from(on_success), PatchResponse::from)
}

/// Return a [`ResponseResult`] for when a [`User`] tries to GET something, but they *effectively*
/// have no permissions (rather than outright having no permissions).
#[allow(clippy::unnecessary_wraps)]
fn no_effective_perms<R>(action: Action, object: Object, reason: Reason) -> ResponseResult<R>
where
	R: AsRef<Code> + From<Status>,
{
	Ok(Response::from(R::from(Status::new(
		Code::SuccessForPermissions,
		format!("This user has permission to {action} {object}, but {reason}"),
	))))
}

const fn todo(msg: &'static str) -> (StatusCode, &'static str)
{
	(StatusCode::NOT_IMPLEMENTED, msg)
}

/// Create routes which are able to be implemented generically.
macro_rules! route {
	($Entity:ident, $Args:ty, $($param:ident),+) => {
		routing::delete(
				|Extension(user): Extension<User>,
				 State(state): State<ServerState<A::Db>>,
				 Json(request): Json<request::Delete<<A::$Entity as Deletable>::Entity>>| async move {
					state.enforce_permission(&user, Object::$Entity, Action::Delete).await?;
					delete::<A::$Entity>(state.pool(), request.into_entities(), Code::Success).await
				},
			)
			.get(
				|Extension(user): Extension<User>,
				 State(state): State<ServerState<A::Db>>,
				 Json(request): Json<request::Get<<A::$Entity as Retrievable>::Match>>| async move {
					state.enforce_permission(&user, Object::$Entity, Action::Retrieve).await?;
					retrieve::<A::$Entity>(state.pool(), request.into_condition(), Code::Success).await
				},
			)
			.patch(
				|Extension(user): Extension<User>,
				 State(state): State<ServerState<A::Db>>,
				 Json(request): Json<request::Patch<<A::$Entity as Deletable>::Entity>>| async move {
					state.enforce_permission(&user, Object::$Entity, Action::Update).await?;
					update::<A::$Entity>(state.pool(), request.into_entities(), Code::Success).await
				},
			)
			.post(
				|Extension(user): Extension<User>,
				 State(state): State<ServerState<A::Db>>,
				 Json(request): Json<request::Post<$Args>>| async move {
					state.enforce_permission(&user, Object::$Entity, Action::Create).await?;
					let ( $($param),+ ) = request.into_args();
					create(A::$Entity::create(state.pool(), $($param),+).await, Code::Success)
				},
			)
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
	for<'con> &'con mut <A::Db as Database>::Connection: Executor<'con, Database = A::Db>,
{
	/// The handler for the [`routes::CONTACT`](crate::api::routes::CONTACT).
	pub fn contact(&self) -> MethodRouter<ServerState<A::Db>>
	{
		route!(Contact, (ContactKind, String), kind, name)
	}

	/// The handler for the [`routes::DEPARTMENT`](crate::api::routes::DEPARTMENT).
	pub fn department(&self) -> MethodRouter<ServerState<A::Db>>
	{
		routing::delete(
			|Extension(user): Extension<User>,
			 State(state): State<ServerState<A::Db>>,
			 Json(request): Json<request::Delete<Department>>| async move {
				let entities = request.into_entities();
				let code = match state.department_permissions(&user, Action::Delete).await?
				{
					Object::Department => Code::Success,

					// they have no department, so they *effectively* can't retrieve departments.
					p @ Object::AssignedDepartment =>
					{
						return no_effective_perms(Action::Delete, p, Reason::ResourceExists)
							.map_all(Into::into, Into::into);
					},

					p => p.unreachable(),
				};

				delete::<A::Department>(state.pool(), entities, code).await
			},
		)
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

					p @ Object::AssignedDepartment =>
					{
						return no_effective_perms(Action::Retrieve, p, Reason::NoDepartment)
					},
					p => p.unreachable(),
				};

				retrieve::<A::Department>(state.pool(), condition, code).await
			},
		)
		.patch(
			|Extension(user): Extension<User>,
			 State(state): State<ServerState<A::Db>>,
			 Json(request): Json<request::Patch<Department>>| async move {
				let mut entities = request.into_entities();
				let code = match state.department_permissions(&user, Action::Update).await?
				{
					Object::Department => Code::Success,

					// HACK: no if-let guards…
					Object::AssignedDepartment if user.employee().is_some() =>
					{
						let id = user.employee().unwrap().department.id;
						entities.retain(|d| d.id == id);
						Code::SuccessForPermissions
					},

					// they have no department, so they *effectively* can't retrieve departments.
					p @ Object::AssignedDepartment =>
					{
						return no_effective_perms(Action::Retrieve, p, Reason::NoDepartment)
							.map_all(Into::into, Into::into)
					},

					p => p.unreachable(),
				};

				update::<A::Department>(state.pool(), entities, code).await
			},
		)
		.post(
			|Extension(user): Extension<User>,
			 State(state): State<ServerState<A::Db>>,
			 Json(request): Json<request::Post<String>>| async move {
				let name = request.into_args();
				let code = match state.department_permissions(&user, Action::Create).await?
				{
					Object::Department => Code::Success,

					p @ Object::AssignedDepartment =>
					{
						return no_effective_perms(Action::Retrieve, p, Reason::ResourceConstraint);
					},
					p => p.unreachable(),
				};

				create(A::Department::create(state.pool(), name).await, code)
			},
		)
	}

	/// The handler for the [`routes::EMPLOYEE`](crate::api::routes::EMPLOYEE).
	pub fn employee(&self) -> MethodRouter<ServerState<A::Db>>
	{
		routing::delete(
			|Extension(user): Extension<User>,
			 State(state): State<ServerState<A::Db>>,
			 Json(request): Json<request::Delete<Employee>>| async move {
				let mut entities = request.into_entities();
				let code = match state.employee_permissions(&user, Action::Delete).await?
				{
					Object::Employee => Code::Success,

					// HACK: no if-let guards…
					Object::EmployeeInDepartment if user.employee().is_some() =>
					{
						let id = user.employee().unwrap().department.id;
						entities.retain(|e| e.department.id == id);
						Code::SuccessForPermissions
					},

					Object::EmployeeSelf => return Err(DeleteResponse::from(Code::Unauthorized)),
					p @ Object::EmployeeInDepartment =>
					{
						return no_effective_perms(Action::Delete, p, Reason::NoDepartment)
							.map_all(Into::into, Into::into)
					},

					p => p.unreachable(),
				};

				delete::<A::Employee>(state.pool(), entities, code).await
			},
		)
		.get(
			|Extension(user): Extension<User>,
			 State(state): State<ServerState<A::Db>>,
			 Json(request): Json<request::Get<MatchEmployee>>| async move {
				let mut condition = request.into_condition();
				let code = match state.employee_permissions(&user, Action::Retrieve).await?
				{
					Object::Employee => Code::Success,

					// HACK: no if-let guards…
					Object::EmployeeInDepartment if user.employee().is_some() =>
					{
						condition.department.id &= user.employee().unwrap().department.id.into();
						Code::SuccessForPermissions
					},

					// HACK: no if-let guards…
					Object::EmployeeSelf if user.employee().is_some() =>
					{
						condition.id &= user.employee().unwrap().id.into();
						Code::SuccessForPermissions
					},

					p @ Object::EmployeeInDepartment =>
					{
						return no_effective_perms(Action::Retrieve, p, Reason::NoDepartment)
					},

					p @ Object::EmployeeSelf =>
					{
						return no_effective_perms(Action::Update, p, Reason::NoEmployee);
					},

					p => p.unreachable(),
				};

				retrieve::<A::Employee>(state.pool(), condition, code).await
			},
		)
		.patch(
			|Extension(user): Extension<User>,
			 State(state): State<ServerState<A::Db>>,
			 Json(request): Json<request::Patch<Employee>>| async move {
				let mut entities = request.into_entities();
				let code = match state.employee_permissions(&user, Action::Update).await?
				{
					Object::Employee => Code::Success,

					// HACK: no if-let guards…
					Object::EmployeeInDepartment if user.employee().is_some() =>
					{
						let id = user.employee().unwrap().department.id;
						entities.retain(|e| e.department.id == id);
						Code::SuccessForPermissions
					},

					// HACK: no if-let guards…
					Object::EmployeeSelf if user.employee().is_some() =>
					{
						let id = user.employee().unwrap().id;
						entities.retain(|e| e.id == id);
						Code::SuccessForPermissions
					},

					p @ Object::EmployeeInDepartment =>
					{
						return no_effective_perms(Action::Update, p, Reason::NoDepartment)
							.map_all(Into::into, Into::into)
					},

					p @ Object::EmployeeSelf =>
					{
						return no_effective_perms(Action::Update, p, Reason::NoEmployee)
							.map_all(Into::into, Into::into)
					},

					p => p.unreachable(),
				};

				update::<A::Employee>(state.pool(), entities, code).await
			},
		)
		.post(
			|Extension(user): Extension<User>,
			 State(state): State<ServerState<A::Db>>,
			 Json(request): Json<request::Post<(Department, String, String)>>| async move {
				let (department, name, title) = request.into_args();
				let code = match state.employee_permissions(&user, Action::Create).await?
				{
					Object::Employee => Code::Success,

					// HACK: no if-let guards…
					Object::EmployeeInDepartment
						if user.employee().map_or(false, |e| e.department.id == department.id) =>
					{
						Code::SuccessForPermissions
					},

					p @ Object::EmployeeInDepartment =>
					{
						return no_effective_perms(Action::Create, p, Reason::NoDepartment)
					},

					p @ Object::EmployeeSelf => return no_effective_perms(Action::Create, p, Reason::ResourceExists),
					p => p.unreachable(),
				};

				create(A::Employee::create(state.pool(), department, name, title).await, code)
			},
		)
	}

	/// The handler for the [`routes::EXPENSE`](crate::api::routes::EXPENSE).
	pub fn expense(&self) -> MethodRouter<ServerState<A::Db>>
	{
		async fn retain_matching<A>(
			pool: &Pool<A::Db>,
			employee: &Employee,
			entities: &mut Vec<Expense>,
			permission: Object,
		) -> sqlx::Result<()>
		where
			A: Adapter,
		{
			let matching: HashSet<_> = A::Timesheet::retrieve(pool, MatchTimesheet {
				expenses: MatchExpense {
					id: Match::Or(entities.iter().map(|x| x.id.into()).collect()),
					..Default::default()
				}
				.into(),
				..match permission
				{
					Object::ExpensesInDepartment =>
					{
						MatchJob::from(MatchDepartment::from(employee.department.id)).into()
					},
					Object::CreatedExpenses => MatchEmployee::from(employee.id).into(),
					_ => permission.unreachable(),
				}
			})
			.await
			.map(|vec| vec.into_iter().flat_map(|t| t.expenses.into_iter().map(|x| x.id)).collect())?;

			entities.retain(|x| matching.contains(&x.id));
			Ok(())
		}

		routing::delete(
			|Extension(user): Extension<User>,
			 State(state): State<ServerState<A::Db>>,
			 Json(request): Json<request::Delete<Expense>>| async move {
				let permission = state.expense_permissions(&user, Action::Delete).await?;

				// The user has no department, and no employee record, so they effectively cannot retrieve
				// expenses.
				if permission != Object::Expenses && user.employee().is_none()
				{
					return no_effective_perms(Action::Delete, permission, Reason::NoEmployee)
						.map_all(Into::into, Into::into);
				}

				let mut entities = request.into_entities();

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
								retain_matching::<A>(state.pool(), emp, &mut entities, p)
									.await
									.map_err(DeleteResponse::from)?;
							},

							None => unreachable!("Should have been returned earlier for {permission:?}"),
						};

						Code::SuccessForPermissions
					},
				};

				delete::<A::Expenses>(state.pool(), entities, code).await
			},
		)
		.get(
			|Extension(user): Extension<User>,
			 State(state): State<ServerState<A::Db>>,
			 Json(request): Json<request::Get<MatchExpense>>| async move {
				let permission = state.expense_permissions(&user, Action::Retrieve).await?;

				// The user has no department, and no employee record, so they effectively cannot retrieve
				// expenses.
				if permission != Object::Expenses && user.employee().is_none()
				{
					return no_effective_perms(Action::Retrieve, permission, Reason::NoEmployee);
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
								retain_matching::<A>(state.pool(), emp, &mut vec, p)
									.await
									.map_err(|e| Response::from(Get::from(Status::from(&e))))?;
							},

							None => unreachable!("Should have been returned earlier for {permission:?}"),
						};

						Code::SuccessForPermissions
					},
				};

				Ok::<_, Response<_>>(Response::from(Get::new(vec, code.into())))
			},
		)
		.patch(
			|Extension(user): Extension<User>,
			 State(state): State<ServerState<A::Db>>,
			 Json(request): Json<request::Patch<Expense>>| async move {
				let permission = state.expense_permissions(&user, Action::Update).await?;

				// The user has no department, and no employee record, so they effectively cannot retrieve
				// expenses.
				if permission != Object::Expenses && user.employee().is_none()
				{
					return no_effective_perms(Action::Update, permission, Reason::NoEmployee)
						.map_all(Into::into, Into::into);
				}

				let mut entities = request.into_entities();

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
								retain_matching::<A>(state.pool(), emp, &mut entities, p)
									.await
									.map_err(PatchResponse::from)?;
							},

							None => unreachable!("Should have been returned earlier for {permission:?}"),
						};

						Code::SuccessForPermissions
					},
				};

				update::<A::Expenses>(state.pool(), entities, code).await
			},
		)
		.post(
			#[allow(clippy::type_complexity)]
			|Extension(user): Extension<User>,
			 State(state): State<ServerState<A::Db>>,
			 Json(request): Json<request::Post<(Vec<(String, Money, String)>, Id)>>| async move {
				#[warn(clippy::type_complexity)]
				let permission = state.expense_permissions(&user, Action::Create).await?;

				// The user has no department, and no employee record, so they effectively cannot retrieve
				// expenses.
				if permission != Object::Expenses && user.employee().is_none()
				{
					return no_effective_perms(Action::Create, permission, Reason::NoEmployee);
				}

				let (expenses, timesheet_id) = request.into_args();

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
								let matching: HashSet<_> = A::Timesheet::retrieve(state.pool(), match permission
								{
									Object::ExpensesInDepartment =>
									{
										MatchJob::from(MatchDepartment::from(emp.department.id)).into()
									},
									Object::CreatedExpenses => MatchEmployee::from(emp.id).into(),
									_ => permission.unreachable(),
								})
								.await
								.map_all(
									|vec| vec.into_iter().map(|t| t.id).collect(),
									|e| Response::from(Post::from(Status::from(&e))),
								)?;

								if !matching.contains(&timesheet_id)
								{
									return no_effective_perms(Action::Create, p, Reason::NoResourceExists);
								}
							},

							None => unreachable!("Should have been returned earlier for {permission:?}"),
						};

						Code::SuccessForPermissions
					},
				};

				create(A::Expenses::create(state.pool(), expenses, timesheet_id).await, code)
			},
		)
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

						p @ Object::JobInDepartment =>
						{
							return no_effective_perms(Action::Retrieve, p, Reason::NoDepartment);
						},

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
		route!(Location, (Option<Currency>, String, Option<Location>), currency, name, outer)
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

					auth.login(&user).await.map_all(
						|_| LoginResponse::from(Code::Success),
						|e| {
							const CODE: Code = Code::LoginError;
							tracing::error!("Failed to to log in user {}: {e}", user.username());
							LoginResponse::new(CODE.into(), Status::new(CODE, e.to_string()))
						},
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
	pub const fn new() -> Self
	{
		Self { phantom: PhantomData }
	}

	/// The handler for the [`routes::ORGANIZATION`](crate::api::routes::ORGANIZATION).
	pub fn organization(&self) -> MethodRouter<ServerState<A::Db>>
	{
		route!(Organization, (Location, String), location, name)
	}

	/// The handler for the [`routes::ROLE`](crate::api::routes::ROLE).
	pub fn role(&self) -> MethodRouter<ServerState<A::Db>>
	{
		route!(Role, (String, Option<Duration>), name, password_ttl)
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
					let code = match state.timesheet_permissions(&user, Action::Retrieve).await?
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

						p @ Object::TimesheetInDepartment =>
						{
							return no_effective_perms(Action::Retrieve, p, Reason::NoDepartment);
						},

						p @ Object::CreatedTimesheet =>
						{
							return no_effective_perms(Action::Retrieve, p, Reason::NoEmployee);
						},

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
					let code = match state.user_permissions(&user, Action::Retrieve).await?
					{
						Object::User => Code::Success,

						// HACK: no if-let guards
						Object::UserInDepartment if user.employee().is_some() =>
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

						Object::UserSelf =>
						{
							condition.id &= user.id().into();
							Code::SuccessForPermissions
						},

						p @ Object::UserInDepartment =>
						{
							return no_effective_perms(Action::Retrieve, p, Reason::NoDepartment);
						},

						p => p.unreachable(),
					};

					retrieve::<A::User>(state.pool(), condition, code).await
				},
			)
			.patch(|| async move { todo("Update method not implemented") })
			.post(|| async move { todo("user create") })
	}
}
