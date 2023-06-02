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
	static URL: OnceLock<String> = OnceLock::new();
	Pool::connect_lazy(&URL.get_or_init(|| dotenvy::var("DATABASE_URL").unwrap())).unwrap()
}

/// Create a string which is guaranteed to be different from `s`.
#[cfg(test)]
pub(crate) fn different_string(s: &str) -> String
{
	format!("!{s}")
}

/// Generate a [`rand::random`] [`String`] of [`len`](String::len) `8`.
#[cfg(test)]
pub(crate) fn random_string() -> String
{
	rand::random::<[char; 8]>().into_iter().collect()
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
