//! Contains implementations of [`SessionStore`] for [`DbSessionStore`] per database.

use axum_login::axum_sessions::async_session::{chrono::Utc, Result, Session, SessionStore};
use sqlx::{types::Json};
use tracing::instrument;

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
		#[instrument(level = "trace", skip(self), err)]
		async fn clear_store(&self) -> Result
		{
			sqlx::query!("TRUNCATE sessions;").execute(&self.pool).await?;
			Ok(())
		}

		#[instrument(level = "trace", skip(self), err)]
		async fn destroy_session(&self, session: Session) -> Result
		{
			sqlx::query!("DELETE FROM sessions WHERE id = $1;", session.id())
				.execute(&self.pool)
				.await?;

			Ok(())
		}

		#[instrument(level = "trace", skip(self), err)]
		async fn load_session(&self, cookie_value: String) -> Result<Option<Session>>
		{
			let id = Session::id_from_cookie_value(&cookie_value)?;
			let row = sqlx::query!(
				r#"SELECT session as "session!: Json<Session>" FROM sessions WHERE id = $1 AND (expires IS NULL OR expires > $2);"#,
				id,
				Utc::now(),
			)
			.fetch_optional(&self.pool)
			.await?;

			Ok(row.map(|r| r.session.0))
		}

		#[instrument(level = "trace", skip(self), err)]
		async fn store_session(&self, session: Session) -> Result<Option<String>>
		{
			let query = sqlx::query!(
				"INSERT INTO sessions (id, session, expires) VALUES ($1, $2, $3) ON CONFLICT(id) \
				 DO UPDATE SET expires = EXCLUDED.expires, session = EXCLUDED.session",
				session.id(),
				Json(&session) as _,
				session.expiry()
			);

			query.execute(&self.pool).await?;
			Ok(session.into_cookie_value())
		}
	}

	#[cfg(test)]
	mod tests
	{
		use super::{DbSessionStore, Postgres, Session, SessionStore};
		use crate::{
			dyn_result::DynResult,
			utils::{connect_pg, random_string},
		};

		/// # Returns
		///
		/// `(session, store)`.
		async fn setup() -> DynResult<(Session, DbSessionStore<Postgres>)>
		{
			let mut session = Session::new();
			session.insert(&random_string(), random_string())?;

			let store = DbSessionStore::new(connect_pg()).await?;
			store.store_session(session.clone()).await?;
			Ok((session, store))
		}

		async fn tear_down(store: &DbSessionStore<Postgres>, session: Session) -> DynResult<()>
		{
			store.destroy_session(session).await?;
			Ok(())
		}

		#[tokio::test]
		async fn clear_store() -> DynResult<()>
		{
			let (session, store) = setup().await?;

			todo!()
		}

		#[tokio::test]
		async fn destroy_session() -> DynResult<()>
		{
			let (session, store) = setup().await?;

			todo!()
		}

		#[tokio::test]
		async fn load_session() -> DynResult<()>
		{
			let (session, store) = setup().await?;

			todo!()
		}

		#[tokio::test]
		async fn store_session() -> DynResult<()>
		{
			let (session, store) = setup().await?;

			todo!()
		}
	}
}
