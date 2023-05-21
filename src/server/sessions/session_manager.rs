//! Contains code which manages connections, log-ins, etc.

mod clone;

use core::time::Duration;
use std::{collections::HashMap, sync::Arc};

use axum::{http::StatusCode, response::IntoResponse};
use sqlx::{pool::PoolOptions, Connection, Database, Error, Executor, Pool, Transaction};
use tokio::sync::RwLock;
use uuid::Uuid;

use super::{session::Session, Login};
use crate::{
	api::{response, StatusCode as WinvoiceCode},
	server::response::Response,
};

type SyncUuidMap<T> = Arc<RwLock<HashMap<Uuid, T>>>;

/// A manager for active
#[derive(Debug)]
pub struct SessionManager<Db>
where
	Db: Database,
{
	/// The base options to create new connections with the [`Database`].
	connect_options: <Db::Connection as Connection>::Options,

	/// The active connections with the [`Database`].
	connections: SyncUuidMap<Pool<Db>>,

	/// The amount of time that an active connection should be idle before it is shut down.
	idle_timeout: Option<Duration>,

	/// The amount of time that it takes before an active session expires.
	session_expire: Option<Duration>,

	/// The currently logged in users.
	sessions: SyncUuidMap<Session>,
}

impl<Db> SessionManager<Db>
where
	Db: Database,
	<Db::Connection as Connection>::Options: Login + Clone,
	for<'connection> &'connection mut Db::Connection: Executor<'connection, Database = Db>,
	for<'connection> &'connection mut Transaction<'connection, Db>:
		Executor<'connection, Database = Db>,
{
	pub fn new(
		connect_options: <Db::Connection as Connection>::Options,
		idle_timeout: Option<Duration>,
		session_expire: Option<Duration>,
	) -> Self
	{
		Self {
			connect_options,
			connections: Arc::new(RwLock::new(HashMap::new())),
			idle_timeout,
			session_expire,
			sessions: Arc::new(RwLock::new(HashMap::new())),
		}
	}

	/// Validate the `username` and `password` by creating a new [`Pool`] that connects to the
	/// database.
	///
	/// If it success, store the `username` and `password` in a [`Session`].
	pub(super) async fn insert(&self, username: String, password: String) -> impl IntoResponse
	{
		let pool = match PoolOptions::<Db>::new()
			.idle_timeout(self.idle_timeout)
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
			if self.connections.read().await.contains_key(&uuid)
			{
				continue;
			}

			self.connections.write().await.insert(uuid, pool);
			break uuid;
		};

		self.sessions.write().await.insert(uuid, Session::new(username, password));
		Response::new(
			StatusCode::OK,
			response::Login::new(WinvoiceCode::LoggedIn, None, Some(uuid)),
		)
	}

	/// Validate the `username` and `password` by creating a new [`Pool`] that connects to the
	/// database.
	///
	/// If it success, store the `username` and `password` in a [`Session`]
	pub(super) async fn remove(&self, uuid: &str) -> impl IntoResponse
	{
		let parsed = match uuid.parse::<Uuid>()
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

		if self.sessions.write().await.remove(&parsed).is_none()
		{
			return Response::new(
				StatusCode::UNPROCESSABLE_ENTITY,
				response::Logout::new(WinvoiceCode::SessionNotFound, None),
			);
		}

		self.connections.write().await.remove(&parsed);
		Response::new(StatusCode::OK, response::Logout::new(WinvoiceCode::LoggedOut, None))
	}
}
