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
	async fn delete<'entity, Conn, Iter>(connection: &Conn, entities: Iter) -> Result<()>
	where
		Self::Entity: 'entity,
		Iter: Iterator<Item = &'entity Self::Entity> + Send,
		for<'con> &'con Conn: Executor<'con, Database = Self::Db>,
	{
		fn mapper(o: &User) -> PgUuid
		{
			PgUuid::from(o.id())
		}

		// TODO: use `for<'a> |e: &'a User| e.id`
		PgSchema::delete::<_, _, UserColumns>(connection, entities.map(mapper)).await
	}
}
