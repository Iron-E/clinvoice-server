use pretty_assertions::assert_eq;

#[allow(clippy::wildcard_imports)]
use super::*;

#[tokio::test]
#[traced_test]
async fn get() -> DynResult<()>
{
	let TestData { admin, client, grunt, guest, manager, pool } = setup("get").await?;

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
