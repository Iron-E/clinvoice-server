use pretty_assertions::assert_eq;

#[allow(clippy::wildcard_imports)]
use super::*;

#[tokio::test]
#[traced_test]
async fn healthy() -> DynResult<()>
{
	let TestData { admin, client, grunt, guest, manager, pool } = setup("healthy").await?;

	{
		let response = client.get_builder(routes::HEALTHY).send().await;
		assert!(response.status().is_success());
	}

	pool.close().await;

	{
		let response = client.get_builder(routes::HEALTHY).send().await;
		assert!(!response.status().is_success());
	}

	Ok(())
}
