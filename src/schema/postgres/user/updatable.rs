//! Contains an [`Updatable`] implementation for [`PgUser`]

use sqlx::{Postgres, Result, Transaction};
use winvoice_adapter::Updatable;
use winvoice_adapter_postgres::{schema::PgEmployee, PgSchema};

use super::PgUser;
use crate::schema::{columns::UserColumns, postgres::PgRole, User};

#[async_trait::async_trait]
impl Updatable for PgUser
{
	type Db = Postgres;
	type Entity = User;

	#[tracing::instrument(level = "trace", skip_all, err)]
	async fn update<'entity, Iter>(connection: &mut Transaction<Self::Db>, entities: Iter) -> Result<()>
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

		PgSchema::update(&mut *connection, UserColumns::default(), |query| {
			query.push_values(peekable_entities, |mut q, e| {
				q.push_bind(e.employee().map(|emp| emp.id))
					.push_bind(e.id())
					.push_bind(e.password())
					.push_bind(e.password_expires())
					.push_bind(e.role().id())
					.push_bind(e.username());
			});
		})
		.await?;

		let employees = entities.clone().flat_map(User::employee);
		PgEmployee::update(&mut *connection, employees).await?;
		PgRole::update(connection, entities.map(User::role)).await?;
		Ok(())
	}
}
