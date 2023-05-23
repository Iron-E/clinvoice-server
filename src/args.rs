mod command;
#[cfg(feature = "postgres")]
mod postgres;

use core::time::Duration;
use std::{
	error::Error,
	net::SocketAddr,
	path::{Path, PathBuf},
};

use axum_server::tls_rustls::RustlsConfig;
use casbin::{CoreApi, Enforcer};
use clap::Parser;
use command::Command;
use futures::TryFutureExt;
use rand::Rng;
use tokio::task::JoinHandle;
use watchman_client::{
	expr::{Expr, NameTerm},
	fields::NameOnly,
	pdu::SubscribeRequest,
	CanonicalPath,
	Connector,
	SubscriptionData,
};

use crate::{
	dyn_result::{DynError, DynResult},
	lock::{self, Lock},
};

/// Winvoice is a tool to track and generate invoices from the command line. Pass --help for more.
///
/// It is capable of managing information about clients, employees, jobs, timesheets, and exporting
/// the information into the format of your choice.
#[derive(Clone, Debug, Parser)]
#[command(version = "0.1.0-alpha.1")]
pub struct Args
{
	/// The IP and port to bind the Winvoice server to.
	#[arg(default_value = "127.0.0.1:3000", long, short, value_name = "IP:PORT")]
	address: SocketAddr,

	/// The file containing the certificate to use for TLS. Must be in PEM format.
	#[arg(long, short, value_name = "FILE")]
	certificate: PathBuf,

	/// The Winvoice adapter which will be used for this server.
	#[command(subcommand)]
	command: Command,

	/// The amount of time that a [`Database`](sqlx::Database) connection may be held open even
	/// though no activity has occurred.
	#[arg(
		default_value = "30s",
		long,
		short = 'i',
		value_name = "DURATION",
		value_parser = humantime::parse_duration,
	)]
	connection_idle: Duration,

	/// The file containing the key to use for TLS. Must be in PEM format.
	#[arg(long, short, value_name = "FILE")]
	key: PathBuf,

	/// A [`casbin`] model. See [the docs](https://casbin.org/docs/supported-models) for more
	/// information.
	///
	/// If none is passed, the [`DefaultModel`](casbin::DefaultModel)
	///
	/// Should be in the same folder as the `--permissions-policy`.
	#[arg(long, short = 'm', value_name = "FILE")]
	permissions_model: Option<String>,

	/// A [`casbin`] policy. Try [the editor](https://casbin.org/editor).
	#[arg(long, short, value_name = "FILE")]
	permissions_policy: String,

	/// The key which will be used to encrypt sensitive data stored by users. If one is not
	/// provided, a random one will be generated.
	///
	/// TODO: allow changing without restarting the server
	#[arg(long, short = 'S', value_name = "KEY")]
	secret: Option<Vec<u8>>,

	/// The amount of time that a session is valid for.
	#[arg(
		default_value = "4hr",
		long,
		short,
		value_name = "DURATION",
		value_parser = humantime::parse_duration,
	)]
	session_ttl: Duration,

	/// The maximum duration to run commands server before timing out (e.g. "5s", "15min").
	///
	/// When this argument is passed without a value (i.e. `--timeout`), a duration of 30 seconds
	/// is set.
	#[arg(
		default_missing_value = "30s",
		num_args = 0..=1,
		long,
		short,
		value_name = "DURATION",
		value_parser = humantime::parse_duration,
	)]
	timeout: Option<Duration>,
}

impl Args
{
	/// Run the Winvoice server.
	pub async fn run(self) -> DynResult<()>
	{
		let model_path = self.permissions_model.map(|s| {
			let l: &'static str = Box::leak(s.into_boxed_str());
			l
		});
		let policy_path: &'static str = Box::leak(self.permissions_policy.into_boxed_str());

		let (permissions, tls) = futures::try_join!(
			Enforcer::new(model_path, policy_path).map_ok(lock::new).err_into::<DynError>(),
			RustlsConfig::from_pem_file(self.certificate, self.key).err_into::<DynError>(),
		)?;

		watch_permissions(permissions.clone(), model_path, policy_path);

		match self.command
		{
			#[cfg(feature = "postgres")]
			Command::Postgres(p) => p.run(
				self.address,
				self.connection_idle,
				self.secret.unwrap_or_else(|| {
					let mut arr = [0u8; 64];
					rand::thread_rng().fill(&mut arr);
					arr.to_vec()
				}),
				self.session_ttl,
				self.timeout,
				tls,
			),
		}
		.await
	}
}

/// Watch the `model_path` and `policy_path` for changes, reloading the `permissions` when they are
/// changed.
///
/// This allows [`winvoice-server`](crate)'s permissions to be hot-reloaded while the server is
/// running.
fn watch_permissions(
	permissions: Lock<Enforcer>,
	model_path: Option<&'static str>,
	policy_path: &'static str,
) -> JoinHandle<Result<(), Box<dyn Error + Send + Sync>>>
{
	tokio::spawn({
		async move {
			/// Get the [`file_name`](Path::file_name) of the [`str`]
			fn file_name(s: &str) -> PathBuf
			{
				PathBuf::from(Path::new(s).file_name().unwrap())
			}

			let client = Connector::new().connect().await?;

			let path = CanonicalPath::canonicalize(policy_path)?;
			let root = client.resolve_root(path).await?;

			let mut names = NameTerm { paths: vec![file_name(policy_path)], wholename: false };
			if let Some(p) = model_path
			{
				names.paths.push(file_name(p));
			}

			let (mut subscription, _) = client
				.subscribe::<NameOnly>(&root, SubscribeRequest {
					expression: Some(Expr::Name(names)),
					fields: vec!["name"],
					..Default::default()
				})
				.await?;

			loop
			{
				match subscription.next().await?
				{
					SubscriptionData::Canceled =>
					{
						println!("Watchman stopped. Hot reloading of permissions is disabled.");
						break;
					},
					SubscriptionData::FilesChanged(_) =>
					{
						let mut p = permissions.write().await;
						*p = Enforcer::new(model_path, policy_path).await?;
					},
					_ => (),
				}
			}

			Ok(())
		}
	})
}
