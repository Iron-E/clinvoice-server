use pretty_assertions::assert_eq;

#[allow(clippy::wildcard_imports)]
use super::*;

#[tokio::test]
#[traced_test]
async fn who_am_i() -> DynResult<()>
{
	let TestData { admin, client, grunt, guest, manager, pool } = setup("post").await?;
	let users: Vec<_> = [admin, guest, grunt, manager].into_iter().collect();

	for (user, password) in &users
	{
		tracing::trace!("Testing /whoami on {user:#?}");

		client.login(user.username(), password).await;
		let response = client.get_builder(routes::WHO_AM_I).send().await;

		let actual = WhoAmIResponse::from(Response::new(response.status(), response.json::<WhoAmI>().await));
		let expected = WhoAmIResponse::from(user.username().to_owned());

		assert_eq!(actual, expected);
		client.logout().await;
	}

	PgUser::delete(&pool, users.iter().map(|(user, _)| user)).await?;
	PgRole::delete(&pool, users.iter().map(|(user, _)| user.role())).await?;
	Ok(())
}
