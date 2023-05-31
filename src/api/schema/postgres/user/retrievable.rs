//! Contains a [`Retrievable`] implementation for [`User`]

use futures::TryStreamExt;
use sqlx::{Pool, Postgres, QueryBuilder, Result};
use winvoice_adapter::{
	fmt::{sql, As, QueryBuilderExt, TableToSql, WithIdentifier},
	Retrievable,
	WriteWhereClause,
};
use winvoice_adapter_postgres::PgSchema;

use super::PgUser;
use crate::api::{
	r#match::MatchUser,
	schema::{
		columns::{RoleColumns, UserColumns},
		User,
	},
};

/// Implementors of this trait are capable of being retrieved from a [`Database`].
#[async_trait::async_trait]
impl Retrievable for PgUser
{
	type Db = Postgres;
	type Entity = User;
	type Match = MatchUser;

	/// Retrieve all [`User`]s (via `connection`) that match the `match_condition`.
	async fn retrieve(
		connection: &Pool<Postgres>,
		match_condition: Self::Match,
	) -> Result<Vec<Self::Entity>>
	{
		const COLUMNS: UserColumns = UserColumns::default();
		const COLUMNS_SCOPED: UserColumns<WithIdentifier<char, &'static str>> =
			COLUMNS.default_scope();

		const ROLE_COLUMNS: RoleColumns<WithIdentifier<char, &'static str>> =
			RoleColumns::default().default_scope();
		const ROLE_COLUMNS_UNIQUE: RoleColumns<As<WithIdentifier<char, &str>, &str>> =
			ROLE_COLUMNS.r#as(RoleColumns::unique());

		let mut query = QueryBuilder::new(sql::SELECT);

		query
			.push_columns(&COLUMNS_SCOPED)
			.push_more_columns(&ROLE_COLUMNS_UNIQUE)
			.push_default_from::<UserColumns>()
			.push_default_equijoin::<RoleColumns, _, _>(ROLE_COLUMNS.id, COLUMNS_SCOPED.role_id);

		PgSchema::write_where_clause(
			PgSchema::write_where_clause(
				Default::default(),
				UserColumns::DEFAULT_ALIAS,
				&match_condition,
				&mut query,
			),
			RoleColumns::DEFAULT_ALIAS,
			&match_condition.role,
			&mut query,
		);

		query.push(';').build_query_as::<User>().fetch(connection).try_collect().await
	}
}

#[cfg(test)]
mod tests
{
	use pretty_assertions::assert_eq;

	#[tokio::test]
	async fn retrieve()
	{
		todo!("Write test")
	}
}
