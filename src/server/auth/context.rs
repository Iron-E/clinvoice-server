//! Contains the [`auth`](super) [`Context`] which is used by the [server](crate::server).

use axum_login::{extractors::AuthContext, SqlxStore};
use sqlx::Pool;
use winvoice_schema::Id;

use crate::api::schema::User;

/// The [`AuthContext`] used by the [`winvoice_server::server`].
pub type Context<Db> = AuthContext<Id, User, SqlxStore<Pool<Db>, User, Id>, Id>;
