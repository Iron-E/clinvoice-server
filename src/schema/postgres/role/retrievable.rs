//! Contains a [`Retrievable`] implementation for [`Role`]

use futures::{StreamExt, TryStreamExt};
use sqlx::{Pool, Postgres, QueryBuilder, Result};
use winvoice_adapter::{
	fmt::{sql, QueryBuilderExt, TableToSql},
	Retrievable,
	WriteWhereClause,
};
use winvoice_adapter_postgres::PgSchema;

use super::PgRole;
use crate::{
	r#match::MatchRole,
	schema::{columns::RoleColumns, Role},
};

#[async_trait::async_trait]
impl Retrievable for PgRole
{
	type Db = Postgres;
	type Entity = Role;
	type Match = MatchRole;

	#[tracing::instrument(level = "trace", skip(connection), err)]
	async fn retrieve(connection: &Pool<Postgres>, match_condition: Self::Match) -> Result<Vec<Self::Entity>>
	{
		const COLUMNS: RoleColumns = RoleColumns::default();

		let columns = COLUMNS.default_scope();
		let mut query = QueryBuilder::new(sql::SELECT);

		query.push_columns(&columns).push_default_from::<RoleColumns>();

		PgSchema::write_where_clause(Default::default(), RoleColumns::DEFAULT_ALIAS, &match_condition, &mut query);

		tracing::debug!("Generated SQL: {}", query.sql());
		query
			.prepare()
			.fetch(connection)
			.map(|row| row.and_then(|r| Self::row_to_view(&COLUMNS, &r)))
			.try_collect()
			.await
	}
}
