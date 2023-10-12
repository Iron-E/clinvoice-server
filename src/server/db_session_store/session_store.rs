//! Contains implementations of [`SessionStore`] for [`DbSessionStore`] per database.

use axum_login::axum_sessions::async_session::{chrono::Utc, Result, Session, SessionStore};
use sqlx::types::Json;
use tracing::instrument;
use winvoice_schema::chrono::DateTime;

use super::DbSessionStore;

#[cfg(feature = "postgres")]
mod postgres
{
	use sqlx::Postgres;

	#[allow(clippy::wildcard_imports)]
	use super::*;

	#[async_trait::async_trait]
	impl SessionStore for DbSessionStore<Postgres>
	{
		#[instrument(level = "trace", skip_all, err)]
		async fn clear_store(&self) -> Result
		{
			sqlx::query!("TRUNCATE sessions;").execute(&self.pool).await?;
			Ok(())
		}

		#[instrument(level = "trace", skip_all, err)]
		async fn destroy_session(&self, session: Session) -> Result
		{
			sqlx::query!("DELETE FROM sessions WHERE id = $1;", session.id()).execute(&self.pool).await?;

			Ok(())
		}

		#[instrument(level = "trace", skip_all, err)]
		async fn load_session(&self, cookie_value: String) -> Result<Option<Session>>
		{
			let id = Session::id_from_cookie_value(&cookie_value)?;
			let row = sqlx::query!(
				r#"SELECT session as "session!: Json<Session>" FROM sessions WHERE id = $1 AND (expiry IS NULL OR expiry > $2);"#,
				id,
				Utc::now().naive_utc(),
			)
			.fetch_optional(&self.pool)
			.await?;

			Ok(row.map(|r| r.session.0))
		}

		#[instrument(level = "trace", skip_all, err)]
		async fn store_session(&self, session: Session) -> Result<Option<String>>
		{
			sqlx::query!(
				"INSERT INTO sessions (id, session, expiry) VALUES ($1, $2, $3) ON CONFLICT(id) DO UPDATE SET expiry \
				 = EXCLUDED.expiry, session = EXCLUDED.session",
				session.id(),
				Json(&session) as _,
				session.expiry().map(DateTime::naive_utc)
			)
			.execute(&self.pool)
			.await?;

			Ok(session.into_cookie_value())
		}
	}

	#[allow(clippy::std_instead_of_core, clippy::str_to_string)]
	#[cfg(all(feature = "test-postgres", test))]
	mod tests
	{
		use core::time::Duration;

		use mockd::words;
		use pretty_assertions::{assert_eq, assert_str_eq};
		use sqlx::{Error as SqlxError, Executor};
		use tracing_test::traced_test;
		use winvoice_adapter_postgres::{fmt::DateTimeExt, schema::util::connect};

		use super::{DbSessionStore, Postgres, Session, SessionStore};
		use crate::dyn_result::DynResult;

		/// Post a row in the database matching `$id`.
		macro_rules! select {
			($connection:expr, $id:expr) => {
				sqlx::query!("SELECT * FROM sessions WHERE id = $1", $id).fetch_optional($connection).await
			};
		}

		#[tokio::test]
		#[traced_test]
		async fn session_store() -> DynResult<()>
		{
			/// assert session was stored properly
			async fn assert_store_session<'conn, E>(connection: E, session: &Session) -> DynResult<()>
			where
				E: Executor<'conn, Database = Postgres>,
			{
				let session_row = match select!(connection, session.id())
				{
					Ok(Some(s)) => s,
					Ok(None) => return Err(SqlxError::RowNotFound.into()),
					Err(e) => return Err(e.into()),
				};

				let json = serde_json::to_value(session)?;
				assert_eq!(session_row.expiry, session.expiry().map(|d| d.pg_sanitize().naive_utc()));
				assert_eq!(session_row.session, json);
				assert_str_eq!(session_row.id, session.id());

				Ok(())
			}

			let store = DbSessionStore::new(connect());
			store.init().await?;

			let (cookie_value, test_session) = {
				let mut session = Session::new();
				session.insert(&words::word(), words::word())?;

				// needed to separate concern of testing `load` from `store`
				let test_session = session.clone();
				let cookie_value = store.store_session(session).await?.unwrap();
				(cookie_value, test_session)
			};

			// assert initial insert works
			assert_store_session(store.connection(), &test_session).await?;

			let test_session_row = {
				// assert load works
				let mut session_row = store.load_session(cookie_value.clone()).await?.unwrap();
				assert_eq!(session_row, test_session);

				// assert upsert works
				session_row.expire_in(Duration::from_secs(rand::random::<u8>().into()));
				let test_session_row = session_row.clone();

				let new_cookie_value = store.store_session(session_row).await?;
				assert_eq!(new_cookie_value, None);
				assert_store_session(store.connection(), &test_session_row).await?;

				test_session_row
			};

			// assert delete works
			store.destroy_session(test_session_row.clone()).await?;
			{
				let retrieved = select!(store.connection(), test_session_row.id())?;
				assert!(retrieved.is_none(), "Expected `None`, got {retrieved:?}");
			}

			// assert clear store works
			let mut session = Session::new();
			session.insert("key", "value")?;
			store.store_session(session).await?;
			store.clear_store().await?;
			let retrieved = sqlx::query!("SELECT * FROM sessions;").fetch_all(store.connection()).await?;
			assert_eq!(retrieved.len(), 0);

			Ok(())
		}
	}
}
