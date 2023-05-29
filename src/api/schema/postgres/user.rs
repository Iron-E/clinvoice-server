//! Contains a  [`User`](crate::api::schema::User) for the [`Postgres`](sqlx::Postgres) database.

mod deletable;
mod retrievable;
mod updatable;
mod user_adapter;

/// A [`User`](crate::api::schema::User) which has specialized implementations for the
/// [`Postgres`](sqlx::Postgres) database.
pub struct PgUser;
