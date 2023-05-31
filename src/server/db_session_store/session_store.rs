//! Contains implementations of [`SessionStore`] for [`DbSessionStore`] per database.

use async_session::{chrono::Utc, Result, Session, SessionStore};

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
		async fn clear_store(&self) -> Result
		{
			sqlx::query!("TRUNCATE sessions;").execute(&self.pool).await?;
			Ok(())
		}

		async fn destroy_session(&self, session: Session) -> Result
		{
			sqlx::query!("DELETE FROM sessions WHERE id = $1;", session.id())
				.execute(&self.pool)
				.await?;

			Ok(())
		}

		async fn load_session(&self, cookie_value: String) -> Result<Option<Session>>
		{
			let id = Session::id_from_cookie_value(&cookie_value)?;
			let row = sqlx::query!(
			r#"SELECT session as "session!: String" FROM sessions WHERE id = $1 AND (expires IS NULL OR expires > $2)"#,
			id,
			Utc::now(),
		)
		.fetch_optional(&self.pool)
		.await?;

			row.map(|r| serde_json::from_str::<Session>(&r.session)).transpose().map_err(Into::into)
		}

		async fn store_session(&self, session: Session) -> Result<Option<String>>
		{
			let json = serde_json::to_string(&session)?;

			sqlx::query!(
				"INSERT INTO sessions (id, session, expires) VALUES ($1, $2, $3) ON CONFLICT(id) \
				 DO UPDATE SET expires = EXCLUDED.expires, session = EXCLUDED.session",
				session.id(),
				json as _,
				session.expiry()
			)
			.execute(&self.pool)
			.await?;

			Ok(session.into_cookie_value())
		}
	}

	#[cfg(test)]
	mod tests
	{
		#[tokio::test]
		async fn session_store()
		{
			todo!("Write test")
		}
	}
}
