//! Contains code which manages connections, log-ins, etc.

use core::time::Duration;
use std::collections::HashMap;

use axum::{
	http::{Request, StatusCode},
	middleware::{self, Next},
	response::IntoResponse,
	routing,
	Router,
	TypedHeader,
};
use headers::authorization::{Authorization, Basic};
use sqlx::{pool::PoolOptions, Connection, Database, Executor, Pool, Result, Transaction};
use uuid::Uuid;

use super::{session::Session, Login};

/// A manager for active
pub struct SessionManager<Db>
where
	Db: Database,
{
	/// The base options to create new connections with the [`Database`].
	connect_options: <Db::Connection as Connection>::Options,

	/// The active connections with the [`Database`].
	connections: HashMap<Uuid, Pool<Db>>,

	/// The amount of time that an active connection should be idle before it is shut down.
	idle_timeout: Option<Duration>,

	/// The amount of time that it takes before an active session expires.
	session_expire: Option<Duration>,

	/// The currently logged in users.
	sessions: HashMap<Uuid, Session>,
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
			connections: HashMap::new(),
			idle_timeout,
			session_expire,
			sessions: HashMap::new(),
		}
	}

	/// Create a new [`Pool`] which attempts to establish a connection with the database that this
	/// [`Router`] has been instructed to communicate with.
	///
	/// Uses `username` and `password` as credentials for the new connection.
	async fn login(&self, username: &str, password: &str) -> Result<Pool<Db>>
	{
		PoolOptions::new()
			.idle_timeout(self.idle_timeout)
			.max_connections(1)
			.connect_with(self.connect_options.clone().login(username, password))
			.await
	}

	/// Create a route to the `/login` page for `PUT`.
	pub fn route_login(&self, router: Router) -> Router
	{
		router
			.route("/login", routing::put(|| async { (StatusCode::NOT_IMPLEMENTED, "login") }))
			.route_layer(middleware::from_fn(my_middleware))
	}

	/// Create a route to the `/logout` page for `DELETE`.
	pub fn route_logout(&self, router: Router) -> Router
	{
		router
			.route("/logout", routing::delete(|| async { (StatusCode::NOT_IMPLEMENTED, "logout") }))
			.route_layer(middleware::from_fn(my_middleware))
	}
}

async fn my_middleware<TBody>(
	TypedHeader(auth): TypedHeader<Authorization<Basic>>,
	request: Request<TBody>,
	next: Next<TBody>,
) -> impl IntoResponse
{
	// do something with `request`...

	let response = next.run(request).await;

	// do something with `response`...

	response
}
