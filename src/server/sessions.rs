//! `sessions` contains functions and data regarding managing user connections.

mod login;
mod session;
mod session_manager;

use axum::{extract::State, response::IntoResponse, TypedHeader};
use headers::{
	authorization::{Basic, Bearer},
	Authorization,
};
pub use login::Login;
pub use session_manager::SessionManager;
use sqlx::{Connection, Database, Executor, Transaction};

pub async fn login<Db>(
	State(sessions): State<SessionManager<Db>>,
	TypedHeader(auth): TypedHeader<Authorization<Basic>>,
) -> impl IntoResponse
where
	Db: Database,
	<Db::Connection as Connection>::Options: Login + Clone,
	for<'connection> &'connection mut Db::Connection: Executor<'connection, Database = Db>,
	for<'connection> &'connection mut Transaction<'connection, Db>:
		Executor<'connection, Database = Db>,
{
	sessions.insert(auth.username().to_owned(), auth.password().to_owned()).await
}

pub async fn logout<Db>(
	State(sessions): State<SessionManager<Db>>,
	TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
) -> impl IntoResponse
where
	Db: Database,
	<Db::Connection as Connection>::Options: Login + Clone,
	for<'connection> &'connection mut Db::Connection: Executor<'connection, Database = Db>,
	for<'connection> &'connection mut Transaction<'connection, Db>:
		Executor<'connection, Database = Db>,
{
	sessions.remove(auth.token()).await
}
