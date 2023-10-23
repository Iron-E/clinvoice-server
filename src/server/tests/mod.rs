#![allow(clippy::std_instead_of_core, clippy::str_to_string, dead_code, unused_imports)]

#[cfg(feature = "test-postgres")]
mod postgres;

use core::{iter, time::Duration};

use axum_test_helper::TestClient;
use casbin::{CoreApi, Enforcer};
use csv::WriterBuilder;
use futures::{stream, FutureExt, StreamExt, TryFutureExt};
use mockd::{address, company, contact, internet, job, name, password, words};
use money2::{Exchange, ExchangeRates};
use sqlx::Pool;
use test_client_ext::{Method, TestClientExt};
use tracing_test::traced_test;
use winvoice_adapter::{
	schema::{
		ContactAdapter,
		DepartmentAdapter,
		EmployeeAdapter,
		ExpensesAdapter,
		JobAdapter,
		LocationAdapter,
		OrganizationAdapter,
		TimesheetAdapter,
	},
	Deletable,
	Retrievable,
	Updatable,
};
use winvoice_match::{
	Match,
	MatchContact,
	MatchDepartment,
	MatchEmployee,
	MatchExpense,
	MatchJob,
	MatchLocation,
	MatchOrganization,
	MatchTimesheet,
};
use winvoice_schema::{
	chrono,
	chrono::{DateTime, TimeZone, Utc},
	ContactKind,
	Currency,
	Department,
	Expense,
	Invoice,
	Location,
	Money,
};

#[allow(clippy::wildcard_imports)]
use super::*;
use crate::{
	api::{
		request,
		response::{Export, Login, Logout, Post, Put, Version, WhoAmI},
		Code,
		Status,
	},
	lock,
	permissions::{Action, Object},
	r#match::{MatchRole, MatchUser},
	schema::{RoleAdapter, UserAdapter},
	server::response::{LoginResponse, LogoutResponse, Response, WhoAmIResponse},
	utils,
};

const DEFAULT_SESSION_TTL: Duration = Duration::from_secs(60 * 2);
const DEFAULT_TIMEOUT: Option<Duration> = Some(Duration::from_secs(60 * 3));

/// The fields for an [`Contact`](winvoice_schema::Contact)
fn contact_args() -> (ContactKind, String)
{
	(ContactKind::Email(contact::email()), words::sentence(4))
}

/// The fields for an [`Employee`](winvoice_schema::Expense) (without the [`Department`].
fn employee_args(department: &Department) -> (Department, String, String)
{
	(department.clone(), name::full(), job::title())
}

/// The fields for an [`Expense`](winvoice_schema::Expense)
fn expense_args() -> (String, Money, String)
{
	(words::word(), Money::new(20_00, 2, utils::rand_currency()), words::sentence(5))
}

fn fmt_duration(duration: Duration) -> String
{
	humantime::format_duration(duration).to_string()
}

fn job_args() -> (Option<DateTime<Utc>>, DateTime<Utc>, Duration, Invoice, String, String)
{
	(
		None,
		Utc::now(),
		Duration::new(7640, 0),
		Invoice { date: None, hourly_rate: Money::new(20_38, 2, utils::rand_currency()) },
		words::sentence(5),
		words::sentence(5),
	)
}

fn job_args_fmt() -> (Option<DateTime<Utc>>, DateTime<Utc>, String, Invoice, String, String)
{
	let args = job_args();
	(args.0, args.1, fmt_duration(args.2), args.3, args.4, args.5)
}

/// The fields for a [`Location`]
fn location_args() -> (Option<Currency>, String, Option<Location>)
{
	(utils::rand_currency().into(), address::country(), None)
}

/// The fields for a [`Role`]
fn role_args() -> (String, Option<Duration>)
{
	(words::sentence(5), Duration::from_secs(rand::random::<u16>().into()).into())
}

/// The fields for a [`Role`]
fn role_args_fmt() -> (String, Option<String>)
{
	let args = role_args();
	(args.0, args.1.map(fmt_duration))
}

/// The fields for a [`Timesheet`](winvoice_schema::Timesheet)
#[allow(clippy::type_complexity)]
fn timesheet_args() -> (Vec<(String, Money, String)>, DateTime<Utc>, Option<DateTime<Utc>>, String)
{
	let now = Utc::now();
	(Default::default(), now, (now + chrono::Duration::hours(3)).into(), words::sentence(5))
}

#[allow(unused_macros)]
macro_rules! fn_setup {
	($Adapter:ty, $Db:ty, $connect:path, $rand_department_name:path) => {
		/// Setup for the tests.
		///
		/// # Returns
		///
		/// * `(client, pool, admin, admin_password, guest, guest_password)`
		async fn setup(test: &str) -> DynResult<TestData<$Db>>
		{
			let mut role_names = ::std::collections::BTreeSet::new();
			while role_names.len() < 4
			{
				role_names.insert(words::sentence(5));
			}

			let admin_role_name = role_names.pop_last().unwrap();
			let grunt_role_name = role_names.pop_last().unwrap();
			let manager_role_name = role_names.pop_last().unwrap();

			let policy = {
				let mut policy_csv = WriterBuilder::new().has_headers(false).from_writer(Vec::new());
				{
					let mut write = |role: &str, obj: Object| -> csv::Result<()> {
						policy_csv.serialize(("p", role, obj, Action::Create))?;
						policy_csv.serialize(("p", role, obj, Action::Delete))?;
						policy_csv.serialize(("p", role, obj, Action::Retrieve))?;
						policy_csv.serialize(("p", role, obj, Action::Update))?;
						Ok(())
					};

					{
						let mut admin = |obj: Object| -> csv::Result<()> { write(&admin_role_name, obj) };
						admin(Object::Contact)?;
						admin(Object::Department)?;
						admin(Object::Employee)?;
						admin(Object::Expenses)?;
						admin(Object::Job)?;
						admin(Object::Location)?;
						admin(Object::Organization)?;
						admin(Object::Role)?;
						admin(Object::Timesheet)?;
						admin(Object::User)?;
					}

					{
						let mut grunt = |obj: Object| -> csv::Result<()> { write(&grunt_role_name, obj) };
						grunt(Object::CreatedExpenses)?;
						grunt(Object::CreatedTimesheet)?;
					}

					{
						let mut manager = |obj: Object| -> csv::Result<()> { write(&manager_role_name, obj) };
						manager(Object::AssignedDepartment)?;
						manager(Object::EmployeeInDepartment)?;
						manager(Object::ExpensesInDepartment)?;
						manager(Object::JobInDepartment)?;
						manager(Object::TimesheetInDepartment)?;
						manager(Object::UserInDepartment)?;
					}
				}

				policy_csv.serialize(("p", &grunt_role_name, Object::EmployeeSelf, Action::Retrieve))?;
				policy_csv.serialize(("p", &grunt_role_name, Object::EmployeeSelf, Action::Update))?;
				policy_csv.serialize(("p", &grunt_role_name, Object::UserSelf, Action::Retrieve))?;
				policy_csv.serialize(("p", &grunt_role_name, Object::UserSelf, Action::Update))?;

				let inner = policy_csv.into_inner()?;
				String::from_utf8(inner)?
			};

			tracing::debug!("Generated policy: {policy}");

			let (model_path, policy_path) = utils::init_model_and_policy_files(
				&format!("server::{}::{test}", stringify!($Adapter)),
				utils::Model::Rbac.to_string(),
				policy,
			)
			.await
			.map(|(m, p)| -> (&'static str, &'static str) {
				(m.to_string_lossy().into_owned().leak(), p.to_string_lossy().into_owned().leak())
			})?;

			let enforcer = Enforcer::new(model_path, policy_path).await.map(lock::new)?;

			let pool = $connect();
			let server = Server::<$Adapter>::router(
				None,
				utils::cookie_secret(),
				Vec::default(),
				ServerState::<$Db>::new(enforcer, pool.clone()),
				DEFAULT_SESSION_TTL,
				DEFAULT_TIMEOUT,
			)
			.await?;

			let admin_password = password::generate(true, true, true, 8);
			let grunt_password = password::generate(true, true, true, 8);
			let guest_password = password::generate(true, true, true, 8);
			let manager_password = password::generate(true, true, true, 8);
			let manager_department =
				<$Adapter as ::winvoice_adapter::schema::Adapter>::Department::create(&pool, $rand_department_name())
					.await
					.unwrap();

			#[rustfmt::skip]
					let (admin, grunt, guest, manager) = futures::try_join!(
						<$Adapter as ::winvoice_adapter::schema::Adapter>::Department::create(&pool,
					$rand_department_name()
				).and_then(|department|
					<$Adapter as ::winvoice_adapter::schema::Adapter>::Employee::create(&pool,
						department, name::full(), job::title(),
					).and_then(|employee| <$Adapter as Adapter>::Role::create(&pool,
						admin_role_name, Duration::from_secs(60).into(),
					).and_then(|role| <$Adapter as Adapter>::User::create(&pool,
						employee.into(), admin_password.to_owned(), role, internet::username(),
					)))
				),

				<$Adapter as ::winvoice_adapter::schema::Adapter>::Employee::create(&pool,
					manager_department.clone(), name::full(), job::title(),
				).and_then(|employee| <$Adapter as Adapter>::Role::create(&pool,
					grunt_role_name, Duration::from_secs(60).into(),
				).and_then(|role| <$Adapter as Adapter>::User::create(&pool,
					employee.into(), grunt_password.to_owned(), role, internet::username(),
				))),

				<$Adapter as ::winvoice_adapter::schema::Adapter>::Department::create(&pool,
					$rand_department_name()
				).and_then(|department|
					<$Adapter as ::winvoice_adapter::schema::Adapter>::Employee::create(&pool,
						department, name::full(), job::title(),
					).and_then(|employee| <$Adapter as Adapter>::Role::create(&pool,
						role_names.pop_last().unwrap(), Duration::from_secs(60).into(),
					).and_then(|role| <$Adapter as Adapter>::User::create(&pool,
						employee.into(), guest_password.to_owned(), role, internet::username(),
					)))
				),

				<$Adapter as ::winvoice_adapter::schema::Adapter>::Employee::create(&pool,
					manager_department, name::full(), job::title(),
				).and_then(|employee| <$Adapter as Adapter>::Role::create(&pool,
					manager_role_name, Duration::from_secs(60).into(),
				).and_then(|role| <$Adapter as Adapter>::User::create(&pool,
					employee.into(), manager_password.to_owned(), role, internet::username(),
				))),
			)?;

			Ok(TestData {
				client: TestClient::new(server),
				pool,
				admin: (admin, admin_password),
				grunt: (grunt, grunt_password),
				guest: (guest, guest_password),
				manager: (manager, manager_password),
			})
		}

		#[tokio::test]
		#[traced_test]
		async fn rejections() -> DynResult<()>
		{
			let TestData { client, admin, .. } = setup("rejections").await?;

			#[rustfmt::skip]
            stream::iter([
                routes::CONTACT, routes::EMPLOYEE, routes::EXPENSE, routes::JOB, routes::LOCATION,
				routes::LOGOUT, routes::ORGANIZATION, routes::ROLE, routes::TIMESHEET, routes::USER,
			])
			.for_each(|route| async {
				tracing::debug!(r#"Testing "{}" rejections…"#, &*route);

				{// assert request rejected when no API version header.
					let response = client.post(route).send().await;
					assert_eq!(response.status(), StatusCode::from(Code::ApiVersionHeaderMissing));
					assert_eq!(&response.json::<Version>().await, VersionResponse::missing().content());
				}

				if route.ne(routes::LOGOUT)
				{
					{// assert POSTs w/out login are rejected
						let response = client.post_builder(route).send().await;
						assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
					}

					{// assert POSTs w/ wrong body are rejected
						client.login(&admin.0, &admin.1).await;

						let response = client.post_builder(route).body("").send().await;
						assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);

						client.logout().await;
					}
				}
			})
			.await;

			Ok(())
		}
	};
}

pub(crate) use fn_setup;

/// Data used for tests.
struct TestData<Db>
where
	Db: Database,
{
	/// A user with every top-level permissions.
	admin: (User, String),

	/// An HTTP client which can be used to communicate with a local instance of the winvoice server.
	client: TestClient,

	/// A user with mid-level permissions.
	manager: (User, String),

	/// A user with bottom-level permissions.
	grunt: (User, String),

	/// A user with no permissions.
	guest: (User, String),

	/// A connection to the database.
	pool: Pool<Db>,
}
