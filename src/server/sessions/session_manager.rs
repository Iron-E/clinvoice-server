//! Contains code which manages connections, log-ins, etc.

mod clone;

use core::time::Duration;
use std::{collections::HashMap, io, sync::Arc};

use axum::http::StatusCode;
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

	/// Create a new [`Pool`] which attempts to establish a connection with the database that this
	/// [`Router`] has been instructed to communicate with.
	///
	/// Uses `username` and `password` as credentials for the new connection.
	pub(super) async fn new_session(
		&self,
		username: String,
		password: String,
	) -> Result<(), Response<response::Login>>
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
				return Err(Response::new(
					StatusCode::INTERNAL_SERVER_ERROR,
					response::Login::new(WinvoiceCode::BadArguments, None),
				))
			},
			#[cfg(feature = "postgres")]
			Err(Error::Database(e))
				if matches!(
					e.try_downcast_ref::<sqlx::postgres::PgDatabaseError>()
						.and_then(sqlx::postgres::PgDatabaseError::routine),
					Some("auth_failed" | "InitializeSessionUserId")
				) =>
			{
				return Err(Response::new(
					StatusCode::UNPROCESSABLE_ENTITY,
					response::Login::new(WinvoiceCode::InvalidCredentials, None),
				));
			},
			Err(e) =>
			{
				return Err(Response::new(
					StatusCode::INTERNAL_SERVER_ERROR,
					response::Login::new(WinvoiceCode::Other, Some(e.to_string())),
				))
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
		Ok(())
	}
}
