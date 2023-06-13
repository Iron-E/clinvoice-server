//! A [`Deletable`] implementation for [`PgRole`]

use sqlx::{Executor, Postgres, Result};
use winvoice_adapter::Deletable;
use winvoice_adapter_postgres::PgSchema;
use winvoice_schema::Id;

use super::PgRole;
use crate::schema::{columns::RoleColumns, Role};

#[async_trait::async_trait]
impl Deletable for PgRole
{
	type Db = Postgres;
	type Entity = Role;

	#[tracing::instrument(level = "trace", skip_all, err)]
	async fn delete<'connection, 'entity, Conn, Iter>(
		connection: Conn,
		entities: Iter,
	) -> Result<()>
	where
		Self::Entity: 'entity,
		Conn: Executor<'connection, Database = Self::Db>,
		Iter: Iterator<Item = &'entity Self::Entity> + Send,
	{
		const fn mapper(o: &Role) -> Id
		{
			o.id()
		}

		// TODO: use `for<'a> |e: &'a Role| e.id`
		PgSchema::delete::<_, _, RoleColumns>(connection, entities.map(mapper)).await
	}
}
