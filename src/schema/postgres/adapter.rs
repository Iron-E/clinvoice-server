//! Contains an [`Adapter`] implementation for [`PgSchema`].

use winvoice_adapter_postgres::PgSchema;

use super::{PgRole, PgUser};
use crate::schema::Adapter;

impl Adapter for PgSchema
{
	type Role = PgRole;
	type User = PgUser;
}
