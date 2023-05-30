//! Contains an [`Updatable`] implementation for [`PgRole`]

use sqlx::{Postgres, Result, Transaction};
use winvoice_adapter::{schema::columns::RoleColumns, Updatable};
use winvoice_adapter_postgres::PgSchema;

use super::PgRole;
use crate::api::schema::{columns::RoleColumns, Role};

#[async_trait::async_trait]
impl Updatable for PgRole
{
	type Db = Postgres;
	type Entity = Role;

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

		PgSchema::update(connection, RoleColumns::default(), |query| {
			query.push_values(peekable_entities, |mut q, e| {
				q.push_bind(e.id()).push_bind(e.name()).push_bind(e.password_ttl());
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
		schema::{LocationAdapter, RoleAdapter},
		Retrievable,
		Updatable,
	};
	use winvoice_match::MatchRole;

	use crate::schema::{util, PgLocation, PgRole};

	#[tokio::test]
	async fn update()
	{
		todo!("Write test");
	}
}
