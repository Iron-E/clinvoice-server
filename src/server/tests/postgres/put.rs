use pretty_assertions::assert_eq;

#[allow(clippy::wildcard_imports)]
use super::*;

#[tokio::test]
#[traced_test]
async fn put() -> DynResult<()>
{
	let TestData { admin, client, grunt, guest, manager, pool } = setup("put").await?;

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
			let args = job_args_fmt();
			(organization.clone(), args.0, args.1, vec![department.clone()], args.2, args.3, args.4, args.5)
		})
		.await;

	client
		.test_post_unauthorized(routes::JOB, &guest.0, &guest.1, {
			let args = job_args_fmt();
			(organization.clone(), args.0, args.1, vec![department.clone()], args.2, args.3, args.4, args.5)
		})
		.await;

	let job_ = client
		.test_post_success::<PgJob, _>(&pool, routes::JOB, &admin.0, &admin.1, {
			let args = job_args_fmt();
			(organization.clone(), args.0, args.1, vec![department.clone()], args.2, args.3, args.4, args.5)
		})
		.await;

	#[rustfmt::skip]
	let job2 = client.test_post_success::<PgJob, _>(&pool, routes::JOB, &manager.0, &manager.1, {
			 let args = job_args_fmt();
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

	client
		.test_post_unauthorized(
			routes::EXPENSE,
			&grunt.0,
			&grunt.1,
			(Vec::<()>::new(), timesheet.id, timesheet.time_begin),
		)
		.await;

	client
		.test_post_unauthorized(
			routes::EXPENSE,
			&guest.0,
			&guest.1,
			(Vec::<()>::new(), timesheet.id, timesheet.time_begin),
		)
		.await;

	{
		client.login(&admin.0, &admin.1).await;
		let response = client
			.put_builder(routes::EXPENSE)
			.json(&request::Put::new((vec![expense_args()], timesheet.id, timesheet.time_begin)))
			.send()
			.await;

		let actual = Response::new(response.status(), response.json::<Put<Vec<Expense>>>().await);
		tracing::debug!("\n\nReceived {actual:#?}\n\n");
		let expected = {
			let entity = actual.content().entity().unwrap();
			let row =
				PgExpenses::retrieve(&pool, entity.iter().map(|x| x.id).collect::<Match<_>>().into()).await.unwrap();
			Response::from(Put::new(row.into(), Code::Success.into()))
		};

		assert_eq!(actual, expected);
		client.logout().await;
	}

	{
		client.login(&manager.0, &manager.1).await;
		let response = client
			.put_builder(routes::EXPENSE)
			.json(&request::Put::new((vec![expense_args()], timesheet2.id, timesheet2.time_begin)))
			.send()
			.await;

		let actual = Response::new(response.status(), response.json::<Put<Vec<Expense>>>().await);
		tracing::debug!("\n\nReceived {actual:#?}\n\n");
		let expected = {
			let entity = actual.content().entity().unwrap();
			let row =
				PgExpenses::retrieve(&pool, entity.iter().map(|x| x.id).collect::<Match<_>>().into()).await.unwrap();
			Response::from(Put::new(row.into(), Code::Success.into()))
		};

		assert_eq!(actual, expected);
		client.logout().await;
	}

	client.test_post_unauthorized(routes::ROLE, &grunt.0, &grunt.1, role_args_fmt()).await;
	client.test_post_unauthorized(routes::ROLE, &guest.0, &guest.1, role_args_fmt()).await;
	client.test_post_unauthorized(routes::ROLE, &manager.0, &manager.1, role_args_fmt()).await;
	let role = client.test_post_success::<PgRole, _>(&pool, routes::ROLE, &admin.0, &admin.1, role_args_fmt()).await;

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
