//! Contains implementations of [`Initializable`] for [`DbSessionStore`].

use sqlx::Acquire;

use super::{DbSessionStore, Initializable, Result};

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
		sqlx::query_file!("src/server/db_session_store/initializable/30-db-session-store.sql").execute(&mut tx).await?;
		tx.commit().await
	}
}
