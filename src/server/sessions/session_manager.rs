//! Contains code which manages connections, log-ins, etc.

mod clone;

use core::time::Duration;
use std::{collections::HashMap, sync::Arc};

use sqlx::{pool::PoolOptions, Connection, Database, Executor, Pool, Transaction};
use tokio::sync::RwLock;
use uuid::Uuid;

use super::{session::Session, Login};
use crate::{api::response, server::response::Response};

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
	/// Create a new [`Pool`] which attempts to establish a connection with the database that this
	/// [`Router`] has been instructed to communicate with.
	///
	/// Uses `username` and `password` as credentials for the new connection.
	pub(super) async fn login(
		&self,
		username: &str,
		password: &str,
	) -> Result<(), Response<response::Login>>
	{
		let pool = match PoolOptions::<Db>::new()
			.idle_timeout(self.idle_timeout)
			.max_connections(1)
			.connect_with(self.connect_options.clone().login(username, password))
			.await
		{
			Ok(p) => p,
			Err(e) => todo!(),
		};

		todo!()
	}

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
}
