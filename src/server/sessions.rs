//! `sessions` contains functions and data regarding managing user connections.

mod login;
mod session;
mod session_manager;

use axum::{extract::State, http::Request, middleware::Next, response::IntoResponse, TypedHeader};
use futures::{FutureExt, TryFutureExt};
use headers::{authorization::Basic, Authorization};
pub use login::Login;
pub use session_manager::SessionManager;
use sqlx::{Connection, Database, Executor, Transaction};

pub async fn login<Db, TBody>(
	State(session_manager): State<SessionManager<Db>>,
	TypedHeader(auth): TypedHeader<Authorization<Basic>>,
	request: Request<TBody>,
	next: Next<TBody>,
) -> impl IntoResponse
where
	Db: Database,
	<Db::Connection as Connection>::Options: Login + Clone,
	for<'connection> &'connection mut Db::Connection: Executor<'connection, Database = Db>,
	for<'connection> &'connection mut Transaction<'connection, Db>:
		Executor<'connection, Database = Db>,
{
	session_manager
		.login(auth.username(), auth.password())
		.map_err(IntoResponse::into_response)
		.then(|_| next.run(request))
		.await
}
