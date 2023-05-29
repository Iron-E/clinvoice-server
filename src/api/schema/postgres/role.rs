//! Contains a  [`Role`](crate::api::schema::Role) for the [`Postgres`](sqlx::Postgres) database.

mod deletable;
mod retrievable;
mod role_adapter;
mod updatable;

/// A [`Role`](crate::api::schema::Role) which has specialized implementations for the
/// [`Postgres`](sqlx::Postgres) database.
pub struct PgRole;
