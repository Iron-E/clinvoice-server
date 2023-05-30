use winvoice_schema::chrono::{DateTime, Datelike, Local, NaiveDateTime, TimeZone, Timelike, Utc};
#[cfg(test)]
use {
	core::cell::OnceCell,
	sqlx::{Database, Pool},
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

/// Connect to the database in the `DATABASE_URL` env variable.
#[cfg(test)]
pub(crate) async fn connect<Db>() -> Pool<Db>
where
	Db: sqlx::Database,
{
	// TODO: use `LazyCell`
	static URL: OnceCell<String> = OnceCell::new();

	Pool::<Db>::connect_lazy(URL.get_or_try_init(|| dotenvy::var("DATABASE_URL")).unwrap()).unwrap()
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
