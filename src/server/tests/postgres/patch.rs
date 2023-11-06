use pretty_assertions::assert_eq;
use winvoice_schema::Employee;

#[allow(clippy::wildcard_imports)]
use super::*;

#[tokio::test]
#[traced_test]
async fn patch() -> DynResult<()>
{
	let TestData { admin, client, mut grunt, guest, manager, pool } = setup("patch").await?;

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

	let grunt_employee = Employee { active: false, ..grunt.0.employee.clone().unwrap() };
	check!(
		PgEmployee, EMPLOYEE;
		manager: [employee, false; manager_employee] => Code::SuccessForPermissions,
		admin: [employee] => None::<Code>,
		grunt: [grunt_employee] => Code::SuccessForPermissions;
		guest,
	);

	grunt.0.employee = grunt_employee.into();
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

	let history = HistoricalExchangeRates::history().await?;

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
		[j, j2]
			.into_iter()
			.map(|jo| {
				HistoricalExchangeRates::exchange_from(&history, Some(jo.date_open.into()), Default::default(), jo)
			})
			.collect::<Vec<_>>()
			.try_into()
			.unwrap()
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
			.map(|ts| {
				let begin = HistoricalExchangeRates::index_ref_from(&history, Some(ts.time_begin.into()));
				let open = HistoricalExchangeRates::index_ref_from(&history, Some(ts.job.date_open.into()));
				ts.exchange_historically(Default::default(), begin, open)
			})
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
			let rates = HistoricalExchangeRates::index_ref_from(&history, Some(t.time_begin.into()));
			PgExpenses::create(&pool, vec![expense_args()], (t.id, t.time_begin)).await.map(|v| {
				x.extend(v.into_iter().map(|mut x| {
					x.category = words::sentence(3);
					x.exchange(Default::default(), &rates)
				}))
			})?;
		}

		x
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
		grunt: [User {password_set: Utc::now().pg_sanitize(), ..grunt.0.clone()}] => Code::SuccessForPermissions;
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
