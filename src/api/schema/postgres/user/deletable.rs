//! A [`Deletable`] implementation for [`PgUser`]

use sqlx::{Executor, Postgres, Result};
use winvoice_adapter::Deletable;
use winvoice_adapter_postgres::PgSchema;
use winvoice_schema::Id;

use super::PgUser;
use crate::api::schema::{columns::UserColumns, User};

#[async_trait::async_trait]
impl Deletable for PgUser
{
	type Db = Postgres;
	type Entity = User;

	async fn delete<'connection, 'entity, Conn, Iter>(
		connection: Conn,
		entities: Iter,
	) -> Result<()>
	where
		Self::Entity: 'entity,
		Conn: Executor<'connection, Database = Self::Db>,
		Iter: Iterator<Item = &'entity Self::Entity> + Send,
	{
		const fn mapper(o: &User) -> Id
		{
			o.id()
		}

		// TODO: use `for<'a> |e: &'a User| e.id`
		PgSchema::delete::<_, _, UserColumns<char>>(connection, entities.map(mapper)).await
	}
}

#[cfg(test)]
mod tests
{
	use pretty_assertions::assert_eq;
	use winvoice_adapter::{
		schema::{LocationAdapter, UserAdapter},
		Deletable,
		Retrievable,
	};
	use winvoice_match::Match;

	use crate::schema::{util, PgLocation, PgUser};

	#[tokio::test]
	async fn delete()
	{
		let connection = util::connect().await;

		// let earth = PgLocation::create(&connection, "Earth".into(), None).await.unwrap();

		// let (organization, organization2, organization3) = futures::try_join!(
		// 	PgUser::create(&connection, earth.clone(), "Some User".into()),
		// 	PgUser::create(&connection, earth.clone(), "Some Other User".into()),
		// 	PgUser::create(&connection, earth.clone(), "Another Other User".into(),),
		// )
		// .unwrap();

		// // The `organization`s still depend on `earth`
		// assert!(PgLocation::delete(&connection, [&earth].into_iter()).await.is_err());
		// PgUser::delete(&connection, [&organization, &organization2].into_iter()).await.unwrap();

		// assert_eq!(
		// 	PgUser::retrieve(
		// 		&connection,
		// 		Match::Or(vec![
		// 			organization.id.into(),
		// 			organization2.id.into(),
		// 			organization3.id.into()
		// 		])
		// 		.into(),
		// 	)
		// 	.await
		// 	.unwrap()
		// 	.as_slice(),
		// 	&[organization3]
		// );
	}
}
