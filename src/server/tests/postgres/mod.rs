mod delete;
mod export;
mod get;
mod patch;
mod post;
mod who_am_i;

use sqlx::Postgres;
use winvoice_adapter_postgres::{
	schema::{
		util::{connect, rand_department_name},
		PgContact,
		PgDepartment,
		PgEmployee,
		PgExpenses,
		PgJob,
		PgLocation,
		PgOrganization,
		PgTimesheet,
	},
	PgSchema,
};

#[allow(clippy::wildcard_imports)]
use super::*;
use crate::schema::postgres::{PgRole, PgUser};

super::fn_setup!(PgSchema, Postgres, connect, rand_department_name);
