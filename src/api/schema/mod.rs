//! Contains extensions to the [`winvoice_schema`] which are specific to the [server](crate).

pub mod columns;
mod role;
#[cfg(feature = "bin")]
mod role_adapter;
mod user;
#[cfg(feature = "bin")]
mod user_adapter;
mod write_where_clause;

pub use role::Role;
pub use user::User;
#[cfg(feature = "postgres")]
pub use {role::PgRole, user::PgUser};
#[cfg(feature = "bin")]
pub use {role_adapter::RoleAdapter, user_adapter::UserAdapter};
