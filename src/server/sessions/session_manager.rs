//! Contains code which manages connections, log-ins, etc.

mod clone;
mod debug;
mod from_ref;

use core::time::Duration;
use std::{collections::HashMap, sync::Arc};

use aes_gcm::{aead::OsRng, Aes256Gcm, KeyInit};
use axum::{http::StatusCode, response::IntoResponse};
use axum_extra::extract::{
	cookie::{Cookie, Key},
	PrivateCookieJar,
};
use sqlx::{pool::PoolOptions, Connection, Database, Executor, Pool, Transaction};
use time::OffsetDateTime;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::{cookie, refresh::Refresh, Login};
use crate::{
	api::{Status, StatusCode as WinvoiceCode, Token},
	server::response::{LoginResponse, LogoutResponse},
};

/// The `session` [`Uuid`] key.
pub const SESSION_ID_KEY: &str = "__winvoice_session_id";

/// The [`Token`] [`Uuid`] key.
pub const TOKEN_KEY: &str = "__winvoice_token";

type SyncMap<K, V> = Arc<RwLock<HashMap<K, V>>>;

/// A manager for active sessions.
pub struct SessionManager<Db>
where
	Db: Database,
{
	/// The base options to create new connections with the [`Database`].
	connect_options: <Db::Connection as Connection>::Options,

	/// The domain to set as the `Domain` field for cookies.
	domain: String,

	/// The [`Key`] that is used to sign all [`Refresh`] [`Token`]s.
	refresh_secret: Key,

	/// The users who may have their `sessions` recreated without logging in again.
	refresh_by_id: SyncMap<Uuid, Refresh>,

	/// The amount of time that it takes before an active session expires.
	refresh_ttl: time::Duration,

	/// The amount of time that an active connection should be idle before it is shut down.
	///
	/// Stored in [`core::time`] format.
	session_idle: Duration,

	/// The `session_ttl_core` but with [`time`] format.
	session_ttl: time::Duration,

	/// The users who are currently connected to the database.
	sessions_by_id: SyncMap<Uuid, Pool<Db>>,
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
	/// [`Database`], the `idle_timeout` for closing inactive connections, and `refresh_ttl` for
	/// pruning old [`Session`]s.
	pub fn new(
		connect_options: <Db::Connection as Connection>::Options,
		domain: String,
		refresh_secret: Key,
		refresh_ttl: time::Duration,
		session_idle: Duration,
		session_ttl: time::Duration,
	) -> Self
	{
		Self {
			connect_options,
			domain,
			refresh_by_id: Arc::new(RwLock::new(HashMap::new())),
			refresh_secret,
			refresh_ttl,
			session_idle,
			session_ttl,
			sessions_by_id: Arc::new(RwLock::new(HashMap::new())),
		}
	}

	/// Validate the `username` and `password` by creating a new [`Pool`](sqlx::Pool) that connects
	/// to the database.
	///
	/// If it success, store the `username` and `password` in a [`Session`].
	pub(super) async fn login(
		&self,
		username: &str,
		password: &str,
		jar: PrivateCookieJar,
	) -> impl IntoResponse
	{
		let pool = match PoolOptions::<Db>::new()
			.idle_timeout(self.session_idle)
			.max_connections(1)
			.connect_with(self.connect_options.clone().login(username, password))
			.await
		{
			Ok(p) => p,
			Err(e) => return Err(LoginResponse::from(&e)),
		};

		let (session_id, refresh_id) =
			futures::join!(new_uuid(&self.sessions_by_id), new_uuid(&self.refresh_by_id),);

		tokio::spawn({
			let pool_clone = pool.clone();
			let sessions = self.sessions_by_id.clone();
			async move {
				pool_clone
					.close_event()
					.do_until(async {}) // HACK: `close_event` doesn't seem to work without `do_until`
					.await
					.ok(); // NOTE: any potential error can be discarded because `do_until` is doing nothing

				sessions.write().await.remove(&session_id);
			}
		});

		let key = Aes256Gcm::generate_key(OsRng);
		let refresh = match Refresh::new(username, password, &key, self.refresh_ttl)
		{
			Ok(e) => e,
			Err(_) =>
			{
				return Err(LoginResponse::new(
					StatusCode::INTERNAL_SERVER_ERROR,
					Status::new(WinvoiceCode::EncryptError, None),
				));
			},
		};

		let refresh_token = Token::new(refresh_id, &key);
		let refresh_cookie = {
			let mut c = cookie::new(
				TOKEN_KEY,
				base64_url::encode(&refresh_token),
				self.domain.clone(),
				refresh.expires(),
			);
			c.set_path("/refresh");
			c
		};

		let session_cookie = cookie::new(
			SESSION_ID_KEY,
			session_id.to_string(),
			self.domain.clone(),
			OffsetDateTime::now_utc() + self.session_ttl,
		);

		futures::join!(
			async { self.sessions_by_id.write().await.insert(session_id, pool) },
			async { self.refresh_by_id.write().await.insert(refresh_id, refresh) },
		);

		Ok((jar.add(session_cookie).add(refresh_cookie), LoginResponse::success()))
	}

	/// TODO: docs
	pub(super) async fn logout(&self, jar: PrivateCookieJar) -> impl IntoResponse
	{
		let mut j = jar;

		// /// Indicates that the `token` parameter that was passed is bad.
		// fn session_not_found() -> Response<response::Logout>
		// {
		// 	Response::new(
		// 		StatusCode::UNPROCESSABLE_ENTITY,
		// 		response::Logout::new(WinvoiceCode::SessionNotFound, None),
		// 	)
		// }

		if let Some(c) = j.get(TOKEN_KEY)
		{
			let decoded = match base64_url::decode(c.value())
			{
				Ok(d) => d,
				Err(e) => return Err(LogoutResponse::from(&e)),
			};

			j = j.remove(Cookie::named(TOKEN_KEY));

			let token = match Token::try_from(decoded.as_slice())
			{
				Ok(t) => t,
				Err(e) => return Err(LogoutResponse::from(&e)),
			};

			todo!(
				"Veryify that the `token.key` matches the encrypted data, to prevent forging a \
				 logout request."
			);

			self.refresh_by_id.write().await.remove(&token.uuid());
		}

		todo!("Destroy session ID");
		Ok((j, LogoutResponse::success()))
	}
}

async fn new_uuid<V>(map: &SyncMap<Uuid, V>) -> Uuid
{
	loop
	{
		let uuid = Uuid::new_v4();
		if map.read().await.contains_key(&uuid)
		{
			continue;
		}

		break uuid;
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
