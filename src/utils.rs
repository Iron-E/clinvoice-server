//! Misc. utilities for [`winvoice_server`] which do not have a more specific category.

use rand::Rng;
use winvoice_schema::chrono::{DateTime, Datelike, Local, NaiveDateTime, TimeZone, Timelike, Utc};
#[cfg(test)]
use {
	core::fmt::{Display, Formatter, Result as FmtResult},
	rand::distributions::Alphanumeric,
	sqlx::Pool,
	std::{env, path::PathBuf, sync::OnceLock},
	tokio::{fs, io::Result as IoResult},
};

/// Create a [`DateTime<Utc>`] out of some [`Local`] [`NaiveDateTime`].
pub fn naive_local_datetime_to_utc(d: NaiveDateTime) -> DateTime<Utc>
{
	Local
		.with_ymd_and_hms(d.year(), d.month(), d.day(), d.hour(), d.minute(), d.second())
		.unwrap()
		.into()
}

/// Connect to the test [`Postgres`](sqlx::Postgres) database.
#[cfg(all(test, feature = "postgres"))]
pub fn connect_pg() -> sqlx::PgPool
{
	static URL: OnceLock<String> = OnceLock::new();
	Pool::connect_lazy(&URL.get_or_init(|| dotenvy::var("DATABASE_URL").unwrap())).unwrap()
}

/// Create a cryptographically-secure, randomly generated key for signing cookies.
pub fn cookie_secret() -> Vec<u8>
{
	let mut arr = [0u8; 64];
	rand::thread_rng().fill(&mut arr);
	arr.to_vec()
}

/// Create a string which is guaranteed to be different from `s`.
#[cfg(test)]
pub fn different_string(s: &str) -> String
{
	format!("!{s}")
}

#[cfg(test)]
pub enum Model
{
	Acl,
	Rbac,
}

#[cfg(test)]
impl Display for Model
{
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult
	{
		match self
		{
			Self::Acl =>
			{
				"[request_definition]
r = sub, obj, act

[policy_definition]
p = sub, obj, act

[policy_effect]
e = some(where (p.eft == allow))

[matchers]
m = r.sub == p.sub && r.obj == p.obj && r.act == p.act
"
			},

			Self::Rbac =>
			{
				"[request_definition]
r = sub, obj, act

[policy_definition]
p = sub, obj, act

[role_definition]
g = _, _

[policy_effect]
e = some(where (p.eft == allow))

[matchers]
m = g(r.sub, p.sub) && r.obj == p.obj && r.act == p.act
"
			},
		}
		.fmt(f)
	}
}

/// # Returns
///
/// * `(model_path, policy_path)`
#[cfg(test)]
pub async fn init_model_and_policy_files<M, P>(
	dir: &str,
	model: M,
	policy: P,
) -> IoResult<(PathBuf, PathBuf)>
where
	M: AsRef<[u8]>,
	P: AsRef<[u8]>,
{
	let temp_dir = temp_dir(dir).await?;
	let model_path = temp_dir.join("model.conf");
	let policy_path = temp_dir.join("policy.csv");

	futures::try_join!(fs::write(&model_path, model), fs::write(&policy_path, policy))?;
	Ok((model_path, policy_path))
}

/// Convert `s` into a `'static` [`str`] by [`Box::leak`]ing it.
/// TODO: use [`String::leak`] (rust-lang/rust#102929)
pub fn leak_string(s: String) -> &'static str
{
	Box::leak(s.into_boxed_str())
}

/// Generate a [`rand::random`] [`String`] of [`len`](String::len) `8`.
#[cfg(test)]
pub fn random_string() -> String
{
	rand::thread_rng().sample_iter(&Alphanumeric).take(8).map(char::from).collect()
}

/// A temporary directory which can be used to write files into for `test`.
#[cfg(test)]
pub(crate) async fn temp_dir(test: &str) -> IoResult<PathBuf>
{
	let mut parent = env::temp_dir();
	parent.push("winvoice-server");
	parent.push(test);

	fs::create_dir_all(&parent).await?;

	Ok(parent)
}
