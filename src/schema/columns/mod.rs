//! Contains extensions to [`winvoice_adapter::schema::columns`] specific to the [server](crate).

mod role;
mod user;

pub use role::RoleColumns;
pub use user::UserColumns;
