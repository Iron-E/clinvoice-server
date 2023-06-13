//! Contains extensions to the [`winvoice_schema`] which are specific to the [server](crate).

#[cfg(feature = "bin")]
mod adapter;
pub mod columns;
#[cfg(feature = "postgres")]
pub mod postgres;
mod role;
#[cfg(feature = "bin")]
mod role_adapter;
mod user;
#[cfg(feature = "bin")]
mod user_adapter;
mod write_where_clause;

pub use role::Role;
pub use user::User;
#[cfg(feature = "bin")]
pub use {adapter::Adapter, role_adapter::RoleAdapter, user_adapter::UserAdapter};
