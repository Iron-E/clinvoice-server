//! Contains a [`Retrievable`] implementation for [`Role`]

use futures::{StreamExt, TryStreamExt};
use sqlx::{Pool, Postgres, QueryBuilder, Result};
use winvoice_adapter::{
	fmt::{sql, QueryBuilderExt, TableToSql, WithIdentifier},
	Retrievable,
	WriteWhereClause,
};
use winvoice_adapter_postgres::PgSchema;

use super::PgRole;
use crate::api::{
	r#match::MatchRole,
	schema::{columns::RoleColumns, Role},
};

/// Implementors of this trait are capable of being retrieved from a [`Database`].
#[async_trait::async_trait]
impl Retrievable for PgRole
{
	type Db = Postgres;
	type Entity = Role;
	type Match = MatchRole;

	/// Retrieve all [`Role`]s (via `connection`) that match the `match_condition`.
	async fn retrieve(
		connection: &Pool<Postgres>,
		match_condition: Self::Match,
	) -> Result<Vec<Self::Entity>>
	{
		const COLUMNS: RoleColumns = RoleColumns::default();
		const COLUMNS_DEFAULT_SCOPE: RoleColumns<WithIdentifier<char, &'static str>> =
			COLUMNS.default_scope();

		let mut query = QueryBuilder::new(sql::SELECT);

		query.push_columns(&COLUMNS_DEFAULT_SCOPE).push_default_from::<RoleColumns>();

		PgSchema::write_where_clause(
			Default::default(),
			RoleColumns::DEFAULT_ALIAS,
			&match_condition,
			&mut query,
		);

		query
			.push(';')
			.build()
			.fetch(connection)
			.map(|row| row.and_then(|r| Self::row_to_view(&COLUMNS, &r)))
			.try_collect()
			.await
	}
}

#[cfg(test)]
mod tests
{
	use std::collections::HashSet;

	use winvoice_adapter::{schema::RoleAdapter, Retrievable};
	use winvoice_match::{Match, MatchRole, MatchStr};

	use crate::schema::{util, PgRole};

	#[tokio::test]
	async fn retrieve()
	{
		todo!("Write test")
	}
}
