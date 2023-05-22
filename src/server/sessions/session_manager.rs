//! Contains code which manages connections, log-ins, etc.

mod clone;

use core::time::Duration;
use std::{collections::HashMap, sync::Arc};

use aes_gcm::{aead::OsRng, Aes256Gcm, KeyInit};
use axum::{http::StatusCode, response::IntoResponse};
use sqlx::{pool::PoolOptions, Connection, Database, Error, Executor, Transaction};
use tokio::sync::RwLock;
use uuid::Uuid;

use super::{session::Session, Login};
use crate::{
	api::{response, StatusCode as WinvoiceCode, Token},
	server::response::Response,
};

/// A manager for active sessions.
#[derive(Debug)]
pub struct SessionManager<Db>
where
	Db: Database,
{
	/// The base options to create new connections with the [`Database`].
	connect_options: <Db::Connection as Connection>::Options,

	/// The amount of time that it takes before an active session expires.
	session_expire: Duration,

	/// The amount of time that an active connection should be idle before it is shut down.
	session_idle: Duration,

	/// The currently logged in users.
	sessions: Arc<RwLock<HashMap<Uuid, Session<Db>>>>,
}

impl<Db> SessionManager<Db>
where
	Db: Database,
	<Db::Connection as Connection>::Options: Login + Clone,
	for<'connection> &'connection mut Db::Connection: Executor<'connection, Database = Db>,
	for<'connection> &'connection mut Transaction<'connection, Db>:
		Executor<'connection, Database = Db>,
{
	/// Create a new [`SessionManager`], which will use `connect_options` for accessing the
	/// [`Database`], the `idle_timeout` for closing inactive connections, and `session_expire` for
	/// pruning old [`Session`]s.
	pub fn new(
		connect_options: <Db::Connection as Connection>::Options,
		session_expire: Duration,
		session_idle: Duration,
	) -> Self
	{
		// TODO: spawn session expirer
		Self {
			connect_options,
			session_idle,
			session_expire,
			sessions: Arc::new(RwLock::new(HashMap::new())),
		}
	}

	/// Validate the `username` and `password` by creating a new [`Pool`](sqlx::Pool) that connects
	/// to the database.
	///
	/// If it success, store the `username` and `password` in a [`Session`].
	pub(super) async fn insert(&self, username: String, password: String) -> impl IntoResponse
	{
		let pool = match PoolOptions::<Db>::new()
			.idle_timeout(self.session_idle)
			.max_connections(1)
			.connect_with(self.connect_options.clone().login(&username, &password))
			.await
		{
			Ok(p) => p,
			Err(Error::Configuration(e)) =>
			{
				return Response::new(
					StatusCode::INTERNAL_SERVER_ERROR,
					response::Login::new(WinvoiceCode::BadArguments, e.to_string().into(), None),
				);
			},
			#[cfg(feature = "postgres")]
			Err(Error::Database(e))
				if matches!(
					e.try_downcast_ref::<sqlx::postgres::PgDatabaseError>()
						.and_then(sqlx::postgres::PgDatabaseError::routine),
					Some("auth_failed" | "InitializeSessionUserId"),
				) =>
			{
				return Response::new(
					StatusCode::UNPROCESSABLE_ENTITY,
					response::Login::new(WinvoiceCode::InvalidCredentials, None, None),
				);
			},
			Err(e) =>
			{
				return Response::new(
					StatusCode::INTERNAL_SERVER_ERROR,
					response::Login::new(WinvoiceCode::Other, e.to_string().into(), None),
				);
			},
		};

		let uuid = loop
		{
			let uuid = Uuid::new_v4();
			if self.sessions.read().await.contains_key(&uuid)
			{
				continue;
			}

			break uuid;
		};

		let key = Aes256Gcm::generate_key(OsRng);
		let session = match Session::new(username, password, &key, pool)
		{
			Ok(e) => e,
			Err(_) =>
			{
				return Response::new(
					StatusCode::INTERNAL_SERVER_ERROR,
					response::Login::new(WinvoiceCode::EncryptError, None, None),
				)
			},
		};

		self.sessions.write().await.insert(uuid, session);
		Response::new(
			StatusCode::OK,
			response::Login::new(WinvoiceCode::LoggedIn, None, Some(Token::new(uuid, &key))),
		)
	}

	/// TODO: docs
	pub(super) async fn remove(&self, token: &[u8]) -> impl IntoResponse
	{
		/// Indicates that the `token` parameter that was passed is bad.
		fn session_not_found() -> Response<response::Logout>
		{
			Response::new(
				StatusCode::UNPROCESSABLE_ENTITY,
				response::Logout::new(WinvoiceCode::SessionNotFound, None),
			)
		}

		let parsed = match Token::try_from(token)
		{
			Ok(p) => p,
			Err(e) =>
			{
				return Response::new(
					StatusCode::BAD_REQUEST,
					response::Logout::new(WinvoiceCode::MalformedUuid, e.to_string().into()),
				)
			},
		};

		self.sessions.write().await.remove(&parsed.uuid()).map_or_else(session_not_found, |_| {
			Response::new(StatusCode::OK, response::Logout::new(WinvoiceCode::LoggedOut, None))
		})
	}
}

#[cfg(test)]
mod tests
{
	#[tokio::test]
	async fn insert_remove()
	{
		todo!("Write this test")
	}
}
