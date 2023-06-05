//! Contains data and algorithms used for authenticating users.

mod initializable_with_authorization;

use axum_login::{extractors::AuthContext as Context, RequireAuthorizationLayer, SqlxStore};
pub use initializable_with_authorization::InitializableWithAuthorization;
use sqlx::Pool;
use winvoice_schema::Id;

use crate::api::schema::User;

/// The [authorization context](Context) used by the [`winvoice_server::server`].
pub type AuthContext<Db> = Context<Id, User, DbUserStore<Db>>;

/// The type which is used to store [`User`]s in the `Db`.
pub type DbUserStore<Db> = SqlxStore<Pool<Db>, User>;

/// The interface used by [`DbUserStore`]
/// HACK: could easily be a type alias in stead rust-lang/types-team#49
pub trait UserStore: axum_login::UserStore<Id, (), User = User> {}
impl<T> UserStore for T where T: axum_login::UserStore<Id, (), User = User> {}

/// A [layer](tower::Layer) for requiring authorization to access a resource.
pub type RequireAuthLayer = RequireAuthorizationLayer<Id, User>;
