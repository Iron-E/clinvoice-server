use pretty_assertions::assert_eq;
use sqlx::Postgres;
use winvoice_adapter_postgres::{
	schema::{
		util::{connect, rand_department_name},
		PgContact,
		PgDepartment,
		PgEmployee,
		PgExpenses,
		PgJob,
		PgLocation,
		PgOrganization,
		PgTimesheet,
	},
	PgSchema,
};

#[allow(clippy::wildcard_imports)]
use super::*;
use crate::schema::postgres::{PgRole, PgUser};

super::fn_setup!(PgSchema, Postgres, connect, rand_department_name);

#[tokio::test]
#[traced_test]
async fn delete() -> DynResult<()>
{
	let TestData { admin, client, grunt, guest, manager, pool } =
		setup("employee_get", DEFAULT_SESSION_TTL, DEFAULT_TIMEOUT).await?;

	macro_rules! check {
		(
			$Adapter:ty, $route:ident;
			$($pass:ident: [ $($data:expr$(, $expected:literal)?);+ $(;)? ] => $code:expr),+ $(,)?;
			$($fail:ident),* $(,)?
		) =>
		{
			$(
				tracing::trace!("Asserting {:?} cannot delete {}", stringify!($fail), stringify!($route));
				client.test_other_unauthorized(Method::Delete, routes::$route, &$fail.0, &$fail.1).await;
			)*

			$({
				tracing::trace!(
					 "\n\n» Asserting {} can delete {}(s) [{}] with Code::{:?}",
					 stringify!($pass),
					 stringify!($route),
					 stringify!($($data$(, ($expected))?);+),
					 $code,
				);

				client.test_other_success::<$Adapter>(
					 Method::Delete,
					 &pool,
					 routes::$route,
					 &$pass.0,
					 &$pass.1,
					 vec![$(( $data.clone(), true$( && $expected)? )),+],
					 $code.into(),
				).await;
			})+
		}
	}

	let contact_ = {
		let (kind, label) = contact_args();
		PgContact::create(&pool, kind, label).await?
	};

	let department = PgDepartment::create(&pool, rand_department_name()).await?;

	let employee = {
		let (d, name_, title) = employee_args(&department);
		PgEmployee::create(&pool, d, name_, title).await?
	};

	let manager_employee = {
		let (d, name_, title) = employee_args(manager.0.department().unwrap());
		PgEmployee::create(&pool, d, name_, title).await?
	};

	let location = {
		let (currency, address_, outer) = location_args();
		PgLocation::create(&pool, currency, address_, outer).await?
	};

	let organization = PgOrganization::create(&pool, location.clone(), company::company()).await?;

	let rates = ExchangeRates::new().await?;

	let [job_, job2]: [_; 2] = {
		let mut tx = pool.begin().await?;
		let (date_close, date_open, increment, invoice, notes, objectives) = job_args();
		let j = PgJob::create(
			&mut tx,
			organization.clone(),
			date_close,
			date_open,
			[department.clone()].into_iter().collect(),
			increment,
			invoice,
			notes,
			objectives,
		)
		.await?;

		let (date_close, date_open, increment, invoice, notes, objectives) = job_args();
		let j2 = PgJob::create(
			&mut tx,
			organization.clone(),
			date_close,
			date_open,
			manager.0.employee().into_iter().map(|e| e.department.clone()).collect(),
			increment,
			invoice,
			notes,
			objectives,
		)
		.await?;

		tx.commit().await?;
		[j, j2].into_iter().map(|jo| jo.exchange(Default::default(), &rates)).collect::<Vec<_>>().try_into().unwrap()
	};

	let [timesheet, timesheet2, timesheet3]: [_; 3] = {
		let mut tx = pool.begin().await?;
		let (expenses, time_begin, time_end, work_notes) = timesheet_args();
		let t =
			PgTimesheet::create(&mut tx, employee.clone(), expenses, job_.clone(), time_begin, time_end, work_notes)
				.await?;

		let (expenses, time_begin, time_end, work_notes) = timesheet_args();
		let t2 = PgTimesheet::create(
			&mut tx,
			grunt.0.employee().unwrap().clone(),
			expenses,
			job2.clone(),
			time_begin,
			time_end,
			work_notes,
		)
		.await?;

		let (expenses, time_begin, time_end, work_notes) = timesheet_args();
		let t3 = PgTimesheet::create(
			&mut tx,
			grunt.0.employee().unwrap().clone(),
			expenses,
			job2.clone(),
			time_begin,
			time_end,
			work_notes,
		)
		.await?;

		tx.commit().await?;
		[t, t2, t3]
			.into_iter()
			.map(|ts| ts.exchange(Default::default(), &rates))
			.collect::<Vec<_>>()
			.try_into()
			.unwrap()
	};

	let expenses = {
		let mut x = Vec::with_capacity(3);
		for t in [&timesheet, &timesheet2, &timesheet3]
		{
			PgExpenses::create(&pool, vec![expense_args()], t.id).await.map(|mut v| x.append(&mut v))?;
		}

		x.exchange(Default::default(), &rates)
	};

	let role = {
		let (name_, password_ttl) = role_args();
		PgRole::create(&pool, name_, password_ttl).await?
	};

	let user = PgUser::create(
		&pool,
		employee.clone().into(),
		password::generate(true, true, true, 8),
		role.clone(),
		internet::username(),
	)
	.await?;

	let manager_user = PgUser::create(
		&pool,
		manager_employee.clone().into(),
		password::generate(true, true, true, 8),
		role.clone(),
		internet::username(),
	)
	.await?;

	let users: Vec<_> = [&admin.0, &guest.0, &grunt.0, &manager.0].into_iter().cloned().collect();
	let roles: Vec<_> = users.iter().map(User::role).collect();

	check!(
		PgUser, USER;
		manager: [user, false; manager_user] => Code::SuccessForPermissions,
		admin: [user] => None::<Code>;
		grunt, guest,
	);
	check!(PgRole, ROLE; admin: [role] => None::<Code>; grunt, guest, manager);
	check!(
		PgExpenses, EXPENSE;
		manager: [expenses[0], false; expenses[2]] => Code::SuccessForPermissions,
		admin: [expenses[0]] => None::<Code>,
		grunt: [expenses[1]] => Code::SuccessForPermissions;
		guest,
	);
	check!(
		PgTimesheet, TIMESHEET;
		manager: [timesheet, false; timesheet3] => Code::SuccessForPermissions,
		admin: [timesheet] => None::<Code>,
		grunt: [timesheet2] => Code::SuccessForPermissions;
		guest,
	);
	check!(
		PgJob, JOB;
		manager: [job_, false; job2] => Code::SuccessForPermissions,
		admin: [job_] => None::<Code>;
		guest, grunt,
	);
	check!(
		PgEmployee, EMPLOYEE;
		manager: [employee, false; manager_employee] => Code::SuccessForPermissions,
		admin: [employee] => None::<Code>;
		guest, grunt,
	);
	check!(PgOrganization, ORGANIZATION; admin: [organization] => None::<Code>; grunt, guest, manager);
	check!(PgContact, CONTACT; admin: [contact_] => None::<Code>; grunt, guest, manager);
	check!(PgLocation, LOCATION; admin: [location] => None::<Code>; grunt, guest, manager);
	check!(
		PgDepartment, DEPARTMENT;
		admin: [department] => None::<Code>;
		guest, grunt, manager,
	);

	PgUser::delete(&pool, users.iter()).await?;
	futures::try_join!(
		PgRole::delete(&pool, roles.into_iter()),
		PgEmployee::delete(&pool, users.iter().filter_map(User::employee)),
	)?;
	PgDepartment::delete(&pool, users.iter().filter_map(User::department)).await?;

	Ok(())
}

#[tokio::test]
#[traced_test]
async fn get() -> DynResult<()>
{
	let TestData { admin, client, grunt, guest, manager, pool } =
		setup("employee_get", DEFAULT_SESSION_TTL, DEFAULT_TIMEOUT).await?;

	macro_rules! assert_unauthorized {
			 ($Match:ty, $route:ident; $($fail:ident),+) => {
				  stream::iter([$(&$fail),+]).for_each(|data|
						client.test_get_unauthorized::<$Match>(routes::$route, &data.0, &data.1)
				  )
				  .await;
			 }
		}

	let contact_ = {
		let (kind, label) = contact_args();
		PgContact::create(&pool, kind, label).await?
	};

	#[rustfmt::skip]
	client.test_get_success(
		routes::CONTACT,
		&admin.0, &admin.1,
		MatchContact::from(contact_.label.clone()),
		[&contact_].into_iter(), None,
	)
	.await;
	assert_unauthorized!(MatchContact, CONTACT; guest, grunt, manager);

	let department = PgDepartment::create(&pool, rand_department_name()).await?;

	#[rustfmt::skip]
	client.test_get_success(
		routes::DEPARTMENT,
		&admin.0, &admin.1,
		MatchDepartment::from(department.id),
		[&department].into_iter(), None,
	)
	.then(|_| client.test_get_success(
		routes::DEPARTMENT,
		&manager.0, &manager.1,
		MatchDepartment::default(),
		manager.0.employee().into_iter().map(|e| &e.department), Code::SuccessForPermissions.into(),
	))
	.await;
	assert_unauthorized!(MatchDepartment, DEPARTMENT; guest, grunt);

	let employee = {
		let (d, name_, title) = employee_args(&department);
		PgEmployee::create(&pool, d, name_, title).await?
	};

	#[rustfmt::skip]
	client.test_get_success(
		routes::EMPLOYEE,
		&admin.0, &admin.1,
		MatchEmployee::from(employee.id),
		[&employee].into_iter(), None,
	)
	.then(|_| client.test_get_success(
		routes::EMPLOYEE,
		&grunt.0, &grunt.1,
		MatchEmployee::default(),
		grunt.0.employee().into_iter(), Code::SuccessForPermissions.into(),
	))
	.then(|_| client.test_get_success(
		routes::EMPLOYEE,
		&manager.0, &manager.1,
		MatchEmployee::default(),
		[&grunt, &manager].into_iter().map(|e| e.0.employee().unwrap()), Code::SuccessForPermissions.into(),
	))
	.await;
	assert_unauthorized!(MatchEmployee, EMPLOYEE; guest);

	let location = {
		let (currency, address_, outer) = location_args();
		PgLocation::create(&pool, currency, address_, outer).await?
	};

	#[rustfmt::skip]
	client.test_get_success(
		routes::LOCATION,
		&admin.0, &admin.1,
		MatchLocation::from(location.id),
		[&location].into_iter(), None,
	)
	.await;
	assert_unauthorized!(MatchLocation, LOCATION; guest, grunt, manager);

	let organization = PgOrganization::create(&pool, location.clone(), company::company()).await?;

	#[rustfmt::skip]
	client.test_get_success(
		routes::ORGANIZATION,
		&admin.0, &admin.1,
		MatchOrganization::from(organization.id),
		[&organization].into_iter(), None,
	)
	.await;
	assert_unauthorized!(MatchOrganization, ORGANIZATION; guest, grunt, manager);

	let rates = ExchangeRates::new().await?;

	let [job_, job2]: [_; 2] = {
		let mut tx = pool.begin().await?;
		let (date_close, date_open, increment, invoice, notes, objectives) = job_args();
		let j = PgJob::create(
			&mut tx,
			organization.clone(),
			date_close,
			date_open,
			[department.clone()].into_iter().collect(),
			increment,
			invoice,
			notes,
			objectives,
		)
		.await?;

		let (date_close, date_open, increment, invoice, notes, objectives) = job_args();
		let j2 = PgJob::create(
			&mut tx,
			organization.clone(),
			date_close,
			date_open,
			manager.0.employee().into_iter().map(|e| e.department.clone()).collect(),
			increment,
			invoice,
			notes,
			objectives,
		)
		.await?;

		tx.commit().await?;
		[j, j2].into_iter().map(|jo| jo.exchange(Default::default(), &rates)).collect::<Vec<_>>().try_into().unwrap()
	};

	#[rustfmt::skip]
	client.test_get_success(
		routes::JOB,
		&admin.0, &admin.1,
		MatchJob::from(job_.id),
		[&job_].into_iter(), None,
	)
	.then(|_| client.test_get_success(
		routes::JOB,
		&manager.0, &manager.1,
		MatchJob::default(),
		[&job2].into_iter(), Code::SuccessForPermissions.into(),
	))
	.await;
	assert_unauthorized!(MatchJob, JOB; guest, grunt);

	let [timesheet, timesheet2, timesheet3]: [_; 3] = {
		let mut tx = pool.begin().await?;
		let (expenses, time_begin, time_end, work_notes) = timesheet_args();
		let t =
			PgTimesheet::create(&mut tx, employee.clone(), expenses, job_.clone(), time_begin, time_end, work_notes)
				.await?;

		let (expenses, time_begin, time_end, work_notes) = timesheet_args();
		let t2 = PgTimesheet::create(
			&mut tx,
			grunt.0.employee().unwrap().clone(),
			expenses,
			job2.clone(),
			time_begin,
			time_end,
			work_notes,
		)
		.await?;

		let (expenses, time_begin, time_end, work_notes) = timesheet_args();
		let t3 = PgTimesheet::create(
			&mut tx,
			manager.0.employee().unwrap().clone(),
			expenses,
			job2.clone(),
			time_begin,
			time_end,
			work_notes,
		)
		.await?;

		tx.commit().await?;
		[t, t2, t3]
			.into_iter()
			.map(|ts| ts.exchange(Default::default(), &rates))
			.collect::<Vec<_>>()
			.try_into()
			.unwrap()
	};

	#[rustfmt::skip]
	client.test_get_success(
		routes::TIMESHEET,
		&admin.0, &admin.1,
		MatchTimesheet::from(timesheet.id),
		[&timesheet].into_iter(), None,
	)
	.then(|_| client.test_get_success(
		routes::TIMESHEET,
		&grunt.0, &grunt.1,
		MatchTimesheet::default(),
		[&timesheet2].into_iter(), Code::SuccessForPermissions.into(),
	))
	.then(|_| client.test_get_success(
		routes::TIMESHEET,
		&manager.0, &manager.1,
		MatchTimesheet::default(),
		[&timesheet2, &timesheet3].into_iter(), Code::SuccessForPermissions.into(),
	))
	.await;
	assert_unauthorized!(MatchTimesheet, TIMESHEET; guest);

	let expenses = {
		let mut x = Vec::with_capacity(2 * 3);
		for t in [&timesheet, &timesheet2, &timesheet3]
		{
			PgExpenses::create(&pool, iter::repeat_with(expense_args).take(2).collect(), t.id)
				.await
				.map(|mut v| x.append(&mut v))?;
		}

		x.exchange(Default::default(), &rates)
	};

	#[rustfmt::skip]
	client.test_get_success(
		routes::EXPENSE,
		&admin.0, &admin.1,
		MatchExpense::from(Match::Or(expenses.iter().map(|x| x.id.into()).collect())),
		expenses.iter(), None,
	)
	.then(|_| client.test_get_success(
		routes::EXPENSE,
		&grunt.0, &grunt.1,
		MatchExpense::default(),
		expenses.iter().filter(|x| x.timesheet_id == timesheet2.id), Code::SuccessForPermissions.into(),
	))
	.then(|_| client.test_get_success(
		routes::EXPENSE,
		&manager.0, &manager.1,
		MatchExpense::default(),
		expenses.iter().filter(|x| x.timesheet_id == timesheet2.id || x.timesheet_id == timesheet3.id),
		Code::SuccessForPermissions.into(),
	))
	.await;
	assert_unauthorized!(MatchExpense, EXPENSE; guest);

	let users: Vec<_> = [&admin.0, &guest.0, &grunt.0, &manager.0].into_iter().cloned().collect();
	let roles: Vec<_> = users.iter().map(|u| u.role().clone()).collect();

	assert_unauthorized!(MatchRole, ROLE; guest, grunt, manager);
	client
		.test_get_success(
			routes::ROLE,
			&admin.0,
			&admin.1,
			MatchRole::from(Match::Or(roles.iter().map(|r| r.id().into()).collect())),
			roles.iter(),
			None,
		)
		.await;

	#[rustfmt::skip]
	client.test_get_success(
		routes::USER,
		&admin.0, &admin.1,
		MatchUser::from(Match::Or(users.iter().map(|u| u.id().into()).collect())),
		users.iter(), None,
	)
	.then(|_| client.test_get_success(
		routes::USER,
		&grunt.0, &grunt.1,
		MatchUser::default(),
		users.iter().filter(|u| u.id() == grunt.0.id()), Code::SuccessForPermissions.into(),
	))
	.then(|_| client.test_get_success(
		routes::USER,
		&manager.0, &manager.1,
		MatchUser::default(),
		users.iter().filter(|u| u.id() == grunt.0.id() || u.id() == manager.0.id()),
			 Code::SuccessForPermissions.into(),
	))
	.await;
	assert_unauthorized!(MatchUser, USER; guest);

	PgUser::delete(&pool, users.iter()).await?;
	futures::try_join!(PgRole::delete(&pool, roles.iter()), PgJob::delete(&pool, [&job_, &job2].into_iter()))?;

	PgOrganization::delete(&pool, [organization].iter()).await?;
	futures::try_join!(
		PgContact::delete(&pool, [&contact_].into_iter()),
		PgEmployee::delete(&pool, users.iter().filter_map(User::employee).chain([&employee])),
		PgLocation::delete(&pool, [&location].into_iter()),
	)?;
	PgDepartment::delete(&pool, users.iter().filter_map(User::department).chain([&employee.department, &department]))
		.await?;

	Ok(())
}

#[tokio::test]
#[traced_test]
async fn patch() -> DynResult<()>
{
	let TestData { admin, client, grunt, guest, manager, pool } =
		setup("employee_get", DEFAULT_SESSION_TTL, DEFAULT_TIMEOUT).await.map(|mut data| {
			data.grunt.0.employee.as_mut().map(|e| e.active = false);
			data.grunt.0.password_expires = None;
			data
		})?;

	macro_rules! check {
		(
			$Adapter:ty, $route:ident;
			$($pass:ident: [ $($data:expr$(, $expected:literal)?);+ $(;)? ] => $code:expr),+ $(,)?;
			$($fail:ident),* $(,)?
		) =>
		{
			$(
				tracing::trace!("Asserting {:?} cannot patch {}", stringify!($fail), stringify!($route));
				client.test_other_unauthorized(Method::Patch, routes::$route, &$fail.0, &$fail.1).await;
			)*

			$({
				tracing::trace!(
					 "\n\n» Asserting {} can patch {}(s) [{}] with Code::{:?}",
					 stringify!($pass),
					 stringify!($route),
					 stringify!($($data$(, $expected)?);+),
					 $code,
				);

				client.test_other_success::<$Adapter>(
					 Method::Patch,
					 &pool,
					 routes::$route,
					 &$pass.0,
					 &$pass.1,
					 vec![$(( $data.clone(), true$( && $expected)? )),+],
					 $code.into(),
				).await;
			})+
		}
	}

	let contact_ = {
		let (kind, label) = contact_args();
		PgContact::create(&pool, kind, label).await.map(|mut c| {
			c.kind = ContactKind::Other(format!("@{}", internet::username()));
			c
		})?
	};

	check!(PgContact, CONTACT; admin: [contact_] => None::<Code>; grunt, guest, manager);

	let department = PgDepartment::create(&pool, rand_department_name()).await.map(|mut d| {
		d.name = words::sentence(7);
		d
	})?;

	check!(
		PgDepartment, DEPARTMENT;
		admin: [department] => None::<Code>,
		manager: [manager.0.department().unwrap()] => Code::SuccessForPermissions;
		guest,
		grunt,
	);

	let employee = {
		let (d, name_, title) = employee_args(&department);
		PgEmployee::create(&pool, d, name_, title).await.map(|mut e| {
			e.name = name::full();
			e
		})?
	};

	let manager_employee = {
		let (d, name_, title) = employee_args(manager.0.department().unwrap());
		PgEmployee::create(&pool, d, name_, title).await.map(|mut e| {
			e.active = !e.active;
			e
		})?
	};

	check!(
		PgEmployee, EMPLOYEE;
		manager: [employee, false; manager_employee] => Code::SuccessForPermissions,
		admin: [employee] => None::<Code>,
		grunt: [grunt.0.employee().unwrap()] => Code::SuccessForPermissions;
		guest,
	);

	let location = {
		let (currency, address_, outer) = location_args();
		PgLocation::create(&pool, currency, address_, outer).await.map(|mut l| {
			l.name = address::street();
			l
		})?
	};

	check!(PgLocation, LOCATION; admin: [location] => None::<Code>; grunt, guest, manager);

	let organization = PgOrganization::create(&pool, location.clone(), company::company()).await.map(|mut o| {
		o.name = words::sentence(4);
		o
	})?;

	check!(PgOrganization, ORGANIZATION; admin: [organization] => None::<Code>; grunt, guest, manager);

	let rates = ExchangeRates::new().await?;

	let [job_, job2]: [_; 2] = {
		let mut tx = pool.begin().await?;
		let (date_close, date_open, increment, invoice, notes, objectives) = job_args();
		let j = PgJob::create(
			&mut tx,
			organization.clone(),
			date_close,
			date_open,
			[department.clone()].into_iter().collect(),
			increment,
			invoice,
			notes,
			objectives,
		)
		.await
		.map(|mut j| {
			j.date_close = (j.date_open + chrono::Duration::days(30)).into();
			j
		})?;

		let (date_close, date_open, increment, invoice, notes, objectives) = job_args();
		let j2 = PgJob::create(
			&mut tx,
			organization.clone(),
			date_close,
			date_open,
			manager.0.employee().into_iter().map(|e| e.department.clone()).collect(),
			increment,
			invoice,
			notes,
			objectives,
		)
		.await
		.map(|mut j| {
			j.date_close = (j.date_open + chrono::Duration::days(30)).into();
			j
		})?;

		tx.commit().await?;
		[j, j2].into_iter().map(|jo| jo.exchange(Default::default(), &rates)).collect::<Vec<_>>().try_into().unwrap()
	};

	check!(
		PgJob, JOB;
		manager: [job_, false; job2] => Code::SuccessForPermissions,
		admin: [job_] => None::<Code>;
		guest, grunt,
	);

	let [timesheet, timesheet2, timesheet3]: [_; 3] = {
		let mut tx = pool.begin().await?;
		let (expenses, time_begin, time_end, work_notes) = timesheet_args();
		let t =
			PgTimesheet::create(&mut tx, employee.clone(), expenses, job_.clone(), time_begin, time_end, work_notes)
				.await
				.map(|mut t| {
					t.time_end = (t.time_begin + chrono::Duration::hours(6)).into();
					t
				})?;

		let (expenses, time_begin, time_end, work_notes) = timesheet_args();
		let t2 = PgTimesheet::create(
			&mut tx,
			grunt.0.employee().unwrap().clone(),
			expenses,
			job2.clone(),
			time_begin,
			time_end,
			work_notes,
		)
		.await
		.map(|mut t| {
			t.time_end = (t.time_begin + chrono::Duration::hours(6)).into();
			t
		})?;

		let (expenses, time_begin, time_end, work_notes) = timesheet_args();
		let t3 = PgTimesheet::create(
			&mut tx,
			manager.0.employee().unwrap().clone(),
			expenses,
			job2.clone(),
			time_begin,
			time_end,
			work_notes,
		)
		.await
		.map(|mut t| {
			t.time_end = (t.time_begin + chrono::Duration::hours(6)).into();
			t
		})?;

		tx.commit().await?;
		[t, t2, t3]
			.into_iter()
			.map(|ts| ts.exchange(Default::default(), &rates))
			.collect::<Vec<_>>()
			.try_into()
			.unwrap()
	};

	check!(
		PgTimesheet, TIMESHEET;
		manager: [timesheet, false; timesheet3] => Code::SuccessForPermissions,
		admin: [timesheet] => None::<Code>,
		grunt: [timesheet2] => Code::SuccessForPermissions;
		guest,
	);

	let expenses = {
		let mut x = Vec::with_capacity(3);
		for t in [&timesheet, &timesheet2, &timesheet3]
		{
			PgExpenses::create(&pool, vec![expense_args()], t.id).await.map(|v| {
				x.extend(v.into_iter().map(|mut x| {
					x.category = words::sentence(3);
					x
				}))
			})?;
		}

		x.exchange(Default::default(), &rates)
	};

	check!(
		PgExpenses, EXPENSE;
		manager: [expenses[0], false; expenses[2]] => Code::SuccessForPermissions,
		admin: [expenses[0]] => None::<Code>,
		grunt: [expenses[1]] => Code::SuccessForPermissions;
		guest,
	);

	let role = {
		let (name_, password_ttl) = role_args();
		PgRole::create(&pool, name_, password_ttl).await.map(|mut r| {
			r.name = words::sentence(7);
			r
		})?
	};

	check!(PgRole, ROLE; admin: [role] => None::<Code>; grunt, guest, manager);

	let user = PgUser::create(&pool, None, password::generate(true, true, true, 8), role.clone(), internet::username())
		.await
		.map(|mut u| {
			u.employee = employee.clone().into();
			u
		})?;

	let manager_user = PgUser::create(
		&pool,
		manager_employee.clone().into(),
		password::generate(true, true, true, 8),
		role.clone(),
		internet::username(),
	)
	.await
	.map(|mut u| {
		u.username = internet::username();
		u
	})?;

	check!(
		PgUser, USER;
		manager: [user, false; manager_user] => Code::SuccessForPermissions,
		admin: [user] => None::<Code>,
		grunt: [grunt.0] => Code::SuccessForPermissions;
		guest,
	);

	let users: Vec<_> = [&admin.0, &guest.0, &grunt.0, &manager.0, &manager_user, &user].into_iter().cloned().collect();

	futures::try_join!(
		PgContact::delete(&pool, [&contact_].into_iter()),
		PgJob::delete(&pool, [&job_, &job2].into_iter()),
		PgUser::delete(&pool, users.iter()),
	)?;

	futures::try_join!(
		PgEmployee::delete(&pool, users.iter().filter_map(User::employee)),
		PgOrganization::delete(&pool, [&organization].into_iter()),
		PgRole::delete(&pool, users.iter().map(User::role)),
	)?;

	futures::try_join!(
		PgDepartment::delete(&pool, users.iter().filter_map(User::department)),
		PgLocation::delete(&pool, [&location].into_iter()),
	)?;

	Ok(())
}

#[tokio::test]
#[traced_test]
async fn post() -> DynResult<()>
{
	let TestData { admin, client, grunt, guest, manager, pool } =
		setup("employee_get", DEFAULT_SESSION_TTL, DEFAULT_TIMEOUT).await?;

	client.test_post_unauthorized(routes::CONTACT, &grunt.0, &grunt.1, contact_args()).await;
	client.test_post_unauthorized(routes::CONTACT, &guest.0, &guest.1, contact_args()).await;
	client.test_post_unauthorized(routes::CONTACT, &manager.0, &manager.1, contact_args()).await;
	let contact_ =
		client.test_post_success::<PgContact, _>(&pool, routes::CONTACT, &admin.0, &admin.1, contact_args()).await;

	client.test_post_unauthorized(routes::DEPARTMENT, &grunt.0, &grunt.1, rand_department_name()).await;
	client.test_post_unauthorized(routes::DEPARTMENT, &guest.0, &guest.1, rand_department_name()).await;
	client.test_post_unauthorized(routes::DEPARTMENT, &manager.0, &manager.1, rand_department_name()).await;
	let department = client
		.test_post_success::<PgDepartment, _>(&pool, routes::DEPARTMENT, &admin.0, &admin.1, rand_department_name())
		.await;

	client.test_post_unauthorized(routes::EMPLOYEE, &grunt.0, &grunt.1, employee_args(&department)).await;
	client.test_post_unauthorized(routes::EMPLOYEE, &guest.0, &guest.1, employee_args(&department)).await;
	let employee = client
		.test_post_success::<PgEmployee, _>(&pool, routes::EMPLOYEE, &admin.0, &admin.1, employee_args(&department))
		.await;

	let manager_employee = client
		.test_post_success::<PgEmployee, _>(
			&pool,
			routes::EMPLOYEE,
			&manager.0,
			&manager.1,
			employee_args(manager.0.department().unwrap()),
		)
		.await;

	client.test_post_unauthorized(routes::LOCATION, &grunt.0, &grunt.1, location_args()).await;
	client.test_post_unauthorized(routes::LOCATION, &guest.0, &guest.1, location_args()).await;
	client.test_post_unauthorized(routes::LOCATION, &manager.0, &manager.1, location_args()).await;
	let location =
		client.test_post_success::<PgLocation, _>(&pool, routes::LOCATION, &admin.0, &admin.1, location_args()).await;

	let organization_args = || (location.clone(), words::sentence(5));

	client.test_post_unauthorized(routes::ORGANIZATION, &grunt.0, &grunt.1, organization_args()).await;
	client.test_post_unauthorized(routes::ORGANIZATION, &guest.0, &guest.1, organization_args()).await;
	client.test_post_unauthorized(routes::ORGANIZATION, &manager.0, &manager.1, organization_args()).await;
	let organization = client
		.test_post_success::<PgOrganization, _>(&pool, routes::ORGANIZATION, &admin.0, &admin.1, organization_args())
		.await;

	client
		.test_post_unauthorized(routes::JOB, &grunt.0, &grunt.1, {
			let args = job_args();
			(organization.clone(), args.0, args.1, vec![department.clone()], args.2, args.3, args.4, args.5)
		})
		.await;

	client
		.test_post_unauthorized(routes::JOB, &guest.0, &guest.1, {
			let args = job_args();
			(organization.clone(), args.0, args.1, vec![department.clone()], args.2, args.3, args.4, args.5)
		})
		.await;

	let job_ = client
		.test_post_success::<PgJob, _>(&pool, routes::JOB, &admin.0, &admin.1, {
			let args = job_args();
			(organization.clone(), args.0, args.1, vec![department.clone()], args.2, args.3, args.4, args.5)
		})
		.await;

	#[rustfmt::skip]
	let job2 = client.test_post_success::<PgJob, _>(&pool, routes::JOB, &manager.0, &manager.1, {
			 let args = job_args();
			 (
				  organization.clone(),
				  args.0, args.1,
				  vec![manager.0.department().unwrap().clone()],
				  args.2, args.3, args.4, args.5,
			 )
		})
		.await;

	client
		.test_post_unauthorized(routes::TIMESHEET, &grunt.0, &grunt.1, {
			let args = timesheet_args();
			(employee.clone(), args.0, job_.clone(), args.1, args.2, args.3)
		})
		.await;

	client
		.test_post_unauthorized(routes::TIMESHEET, &guest.0, &guest.1, {
			let args = timesheet_args();
			(employee.clone(), args.0, job_.clone(), args.1, args.2, args.3)
		})
		.await;

	let timesheet = client
		.test_post_success::<PgTimesheet, _>(&pool, routes::TIMESHEET, &admin.0, &admin.1, {
			let args = timesheet_args();
			(employee.clone(), args.0, job_.clone(), args.1, args.2, args.3)
		})
		.await;

	let timesheet2 = client
		.test_post_success::<PgTimesheet, _>(&pool, routes::TIMESHEET, &manager.0, &manager.1, {
			let args = timesheet_args();
			(grunt.0.employee().unwrap().clone(), args.0, job2.clone(), args.1, args.2, args.3)
		})
		.await;

	client.test_post_unauthorized(routes::EXPENSE, &grunt.0, &grunt.1, (Vec::<()>::new(), timesheet.id)).await;
	client.test_post_unauthorized(routes::EXPENSE, &guest.0, &guest.1, (Vec::<()>::new(), timesheet.id)).await;

	{
		client.login(admin.0.username(), &admin.1).await;
		let response = client
			.post_builder(routes::EXPENSE)
			.json(&request::Post::new((vec![expense_args()], timesheet.id)))
			.send()
			.await;

		let actual = Response::new(response.status(), response.json::<Post<Vec<Expense>>>().await);
		tracing::debug!("\n\nReceived {actual:#?}\n\n");
		let expected = {
			let entity = actual.content().entity().unwrap();
			let row = PgExpenses::retrieve(&pool, Match::from_iter(entity.iter().map(|x| x.id)).into()).await.unwrap();
			Response::from(Post::new(row.into(), Code::Success.into()))
		};

		assert_eq!(actual, expected);
		client.logout().await;
	}

	{
		client.login(manager.0.username(), &manager.1).await;
		let response = client
			.post_builder(routes::EXPENSE)
			.json(&request::Post::new((vec![expense_args()], timesheet2.id)))
			.send()
			.await;

		let actual = Response::new(response.status(), response.json::<Post<Vec<Expense>>>().await);
		tracing::debug!("\n\nReceived {actual:#?}\n\n");
		let expected = {
			let entity = actual.content().entity().unwrap();
			let row = PgExpenses::retrieve(&pool, Match::from_iter(entity.iter().map(|x| x.id)).into()).await.unwrap();
			Response::from(Post::new(row.into(), Code::Success.into()))
		};

		assert_eq!(actual, expected);
		client.logout().await;
	}

	client.test_post_unauthorized(routes::ROLE, &grunt.0, &grunt.1, role_args()).await;
	client.test_post_unauthorized(routes::ROLE, &guest.0, &guest.1, role_args()).await;
	client.test_post_unauthorized(routes::ROLE, &manager.0, &manager.1, role_args()).await;
	let role = client.test_post_success::<PgRole, _>(&pool, routes::ROLE, &admin.0, &admin.1, role_args()).await;

	client
		.test_post_unauthorized(
			routes::USER,
			&grunt.0,
			&grunt.1,
			(Some(employee.clone()), password::generate(true, true, true, 8), role.clone(), internet::username()),
		)
		.await;

	client
		.test_post_unauthorized(
			routes::USER,
			&guest.0,
			&guest.1,
			(Some(employee.clone()), password::generate(true, true, true, 8), role.clone(), internet::username()),
		)
		.await;

	let manager_user = client
		.test_post_success::<PgUser, _>(
			&pool,
			routes::USER,
			&manager.0,
			&manager.1,
			(
				Some(manager_employee.clone()),
				password::generate(true, true, true, 8),
				role.clone(),
				internet::username(),
			),
		)
		.await;

	let user = client
		.test_post_success::<PgUser, _>(
			&pool,
			routes::USER,
			&admin.0,
			&admin.1,
			(Some(employee.clone()), password::generate(true, true, true, 8), role.clone(), internet::username()),
		)
		.await;

	let users: Vec<_> = [&admin.0, &guest.0, &grunt.0, &manager.0, &manager_user, &user].into_iter().cloned().collect();
	let roles: Vec<_> = users.iter().map(User::role).collect();

	PgUser::delete(&pool, users.iter()).await?;
	futures::try_join!(PgJob::delete(&pool, [&job_, &job2].into_iter()), PgRole::delete(&pool, roles.into_iter()),)?;

	futures::try_join!(
		PgEmployee::delete(&pool, [&employee, &manager_employee].into_iter()),
		PgOrganization::delete(&pool, [&organization].into_iter()),
	)?;

	futures::try_join!(
		PgContact::delete(&pool, [&contact_].into_iter()),
		PgDepartment::delete(&pool, [&department].into_iter()),
		PgLocation::delete(&pool, [&location].into_iter()),
	)?;

	Ok(())
}
