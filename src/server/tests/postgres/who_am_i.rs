use pretty_assertions::assert_eq;

#[allow(clippy::wildcard_imports)]
use super::*;

#[tokio::test]
#[traced_test]
async fn who_am_i() -> DynResult<()>
{
	let TestData { admin, client, grunt, guest, manager, pool } = setup("put").await?;
	let users: Vec<_> = [admin, guest, grunt, manager].into_iter().collect();

	for (user, password) in &users
	{
		tracing::trace!("Testing /whoami on {user:#?}");

		client.login(user, password).await;
		let response = client.post_builder(routes::WHO_AM_I).send().await;

		let actual = WhoAmIResponse::from(Response::new(response.status(), response.json::<WhoAmI>().await));
		let expected = WhoAmIResponse::from(user.clone());

		assert_eq!(actual, expected);
		client.logout().await;
	}

	PgUser::delete(&pool, users.iter().map(|(user, _)| user)).await?;
	PgRole::delete(&pool, users.iter().map(|(user, _)| user.role())).await?;
	Ok(())
}
