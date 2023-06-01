//! Misc. utilities for [`winvoice_server`] which do not have a more specific category.

use winvoice_schema::chrono::{DateTime, Datelike, Local, NaiveDateTime, TimeZone, Timelike, Utc};
#[cfg(test)]
use {
	sqlx::Pool,
	std::sync::OnceLock,
	std::{env, fs, path::PathBuf},
};

/// Create a [`DateTime<Utc>`] out of some [`Local`] [`NaiveDateTime`].
pub(crate) fn naive_local_datetime_to_utc(d: NaiveDateTime) -> DateTime<Utc>
{
	Local
		.with_ymd_and_hms(d.year(), d.month(), d.day(), d.hour(), d.minute(), d.second())
		.unwrap()
		.into()
}

/// Connect to the test [`Postgres`](sqlx::Postgres) database.
#[cfg(all(test, feature = "postgres"))]
pub(crate) fn connect_pg() -> sqlx::PgPool
{
	// TODO: use `LazyLock`
	static POOL: OnceLock<sqlx::PgPool> = OnceLock::new();
	POOL.get_or_init(|| Pool::connect_lazy(&dotenvy::var("DATABASE_URL").unwrap()).unwrap()).clone()
}

/// A temporary directory which can be used to write files into for `test`.
#[cfg(test)]
pub(crate) fn temp_dir(test: &str) -> PathBuf
{
	let mut parent = env::temp_dir();
	parent.push("winvoice-server");
	parent.push(test);

	fs::create_dir_all(&parent).unwrap();

	parent
}
