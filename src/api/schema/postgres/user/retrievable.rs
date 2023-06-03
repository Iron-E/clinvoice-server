//! Contains a [`Retrievable`] implementation for [`User`]

use futures::TryStreamExt;
use sqlx::{Pool, Postgres, QueryBuilder, Result};
use winvoice_adapter::{
	fmt::{sql, QueryBuilderExt, TableToSql},
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
	#[tracing::instrument(level = "trace", skip_all, err)]
	async fn retrieve(
		connection: &Pool<Postgres>,
		match_condition: Self::Match,
	) -> Result<Vec<Self::Entity>>
	{
		const COLUMNS: UserColumns = UserColumns::default();
		const ROLE_COLUMNS_UNIQUE: RoleColumns = RoleColumns::unique();

		let columns = COLUMNS.default_scope();
		let role_columns = RoleColumns::default().default_scope();
		let mut query = QueryBuilder::new(sql::SELECT);

		query
			.push_columns(&columns)
			.push_more_columns(&role_columns.r#as(ROLE_COLUMNS_UNIQUE))
			.push_default_from::<UserColumns>()
			.push_default_equijoin::<RoleColumns, _, _>(role_columns.id, columns.role_id);

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

		tracing::debug!("Generated SQL: {}", query.sql());
		query.push(';').build_query_as::<User>().fetch(connection).try_collect().await
	}
}
