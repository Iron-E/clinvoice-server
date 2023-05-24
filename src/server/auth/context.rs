//! Contains the [`auth`](super) [`Context`] which is used by the [server](crate::server).

use axum_login::{extractors::AuthContext, SqlxStore};
use sqlx::Pool;

use super::User;

/// The [`AuthContext`] used by the [`winvoice_server::server`].
pub type Context<Db> = AuthContext<String, User, SqlxStore<Pool<Db>, User, String>, String>;
