//! A [`Deletable`] implementation for [`PgUser`]

use sqlx::{Executor, Postgres, Result};
use winvoice_adapter::Deletable;
use winvoice_adapter_postgres::{fmt::PgUuid, PgSchema};

use super::PgUser;
use crate::schema::{columns::UserColumns, User};

#[async_trait::async_trait]
impl Deletable for PgUser
{
	type Db = Postgres;
	type Entity = User;

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
		fn mapper(o: &User) -> PgUuid
		{
			PgUuid::from(o.id())
		}

		// TODO: use `for<'a> |e: &'a User| e.id`
		PgSchema::delete::<_, _, UserColumns>(connection, entities.map(mapper)).await
	}
}
