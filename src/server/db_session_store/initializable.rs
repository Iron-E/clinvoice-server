//! Contains implementations of [`Initializable`] for [`DbSessionStore`].

use sqlx::Acquire;
use winvoice_adapter::Initializable;

use super::{DbSessionStore, Result};

#[cfg(feature = "postgres")]
#[async_trait::async_trait]
impl Initializable for DbSessionStore<sqlx::Postgres>
{
	type Db = sqlx::Postgres;

	async fn init<'connection, Conn>(connection: Conn) -> Result<()>
	where
		Conn: Acquire<'connection, Database = Self::Db> + Send,
	{
		let mut tx = connection.begin().await?;
		sqlx::query!(
			"CREATE TABLE IF NOT EXISTS sessions
			(
				id text NOT NULL PRIMARY KEY,
				expiry timestamptz,
				session json NOT NULL
			);"
		)
		.execute(&mut tx)
		.await?;

		tx.commit().await
	}
}
