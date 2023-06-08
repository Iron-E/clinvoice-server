//! Contains a  [`User`](crate::api::schema::User) for the [`Postgres`](sqlx::Postgres) database.

mod deletable;
mod from;
mod retrievable;
mod updatable;
mod user_adapter;

/// A [`User`](crate::api::schema::User) which has specialized implementations for the
/// [`Postgres`](sqlx::Postgres) database.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PgUser;
