//! Contains extensions to the [`winvoice_schema`] which are specific to the [server](crate).

pub mod columns;
mod role;
mod user;

pub use role::Role;
pub use user::User;
