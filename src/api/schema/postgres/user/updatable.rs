//! Contains an [`Updatable`] implementation for [`PgUser`]

use sqlx::{Postgres, Result, Transaction};
use winvoice_adapter::{schema::columns::UserColumns, Updatable};
use winvoice_adapter_postgres::PgSchema;
use winvoice_schema::Id;

use super::PgUser;
use crate::api::schema::{columns::UserColumns, User};

#[async_trait::async_trait]
impl Updatable for PgUser
{
	type Db = Postgres;
	type Entity = User;

	async fn update<'entity, Iter>(
		connection: &mut Transaction<Self::Db>,
		entities: Iter,
	) -> Result<()>
	where
		Self::Entity: 'entity,
		Iter: Clone + Iterator<Item = &'entity Self::Entity> + Send,
	{
		let mut peekable_entities = entities.clone().peekable();

		// There is nothing to do.
		if peekable_entities.peek().is_none()
		{
			return Ok(());
		}

		PgSchema::update(connection, UserColumns::default(), |query| {
			query.push_values(peekable_entities, |mut q, e| {
				q.push_bind(e.employee_id())
					.push_bind(e.id())
					.push_bind(e.password())
					.push_bind(e.password_expires())
					.push_bind(e.role())
					.push_bind(e.username());
			});
		})
		.await
	}
}

#[cfg(test)]
mod tests
{
	use pretty_assertions::assert_eq;
	use winvoice_adapter::{
		schema::{LocationAdapter, UserAdapter},
		Retrievable,
		Updatable,
	};
	use winvoice_match::MatchUser;

	use crate::schema::{util, PgLocation, PgUser};

	#[tokio::test]
	async fn update()
	{
		todo!("Write test");
	}
}
