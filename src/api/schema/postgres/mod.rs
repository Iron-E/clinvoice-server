//! Contains [`schema`](super) extensions for [`Postgres`](sqlx::Postgres)

mod adapter;
mod role;
mod user;

pub use role::PgRole;
pub use user::PgUser;
