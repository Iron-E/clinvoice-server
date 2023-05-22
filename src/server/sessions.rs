//! `sessions` contains functions and data regarding managing user connections.

pub mod cookie;
mod login;
mod refresh;
mod session_manager;

use axum::{
	extract::State,
	headers::{
		authorization::{Basic, Bearer},
		Authorization,
	},
	response::IntoResponse,
	TypedHeader,
};
use axum_extra::extract::PrivateCookieJar;
pub use login::Login;
pub use session_manager::{SessionManager, SESSION_ID_KEY, TOKEN_KEY};
use sqlx::{Connection, Database, Executor, Transaction};

pub async fn login<Db>(
	State(sessions): State<SessionManager<Db>>,
	TypedHeader(auth): TypedHeader<Authorization<Basic>>,
	jar: PrivateCookieJar,
) -> impl IntoResponse
where
	Db: Database,
	<Db::Connection as Connection>::Options: Login + Clone,
	for<'connection> &'connection mut Db::Connection: Executor<'connection, Database = Db>,
	for<'connection> &'connection mut Transaction<'connection, Db>:
		Executor<'connection, Database = Db>,
{
	sessions.login(auth.username(), auth.password(), jar).await
}

pub async fn logout<Db>(
	State(_sessions): State<SessionManager<Db>>,
	TypedHeader(_auth): TypedHeader<Authorization<Bearer>>,
) -> impl IntoResponse
where
	Db: Database,
	<Db::Connection as Connection>::Options: Login + Clone,
	for<'connection> &'connection mut Db::Connection: Executor<'connection, Database = Db>,
	for<'connection> &'connection mut Transaction<'connection, Db>:
		Executor<'connection, Database = Db>,
{
	todo!("Write custom `Bearer` impl that allows for invalid UTF-8")
	// sessions.remove(auth.token()).await
}
