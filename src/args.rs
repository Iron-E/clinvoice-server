mod command;
#[cfg(feature = "postgres")]
mod postgres;

use core::time::Duration;
use std::{net::SocketAddr, path::PathBuf};

use axum_extra::extract::cookie::Key;
use axum_server::tls_rustls::RustlsConfig;
use clap::Parser;
use command::Command;

use crate::DynResult;

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

	/// The IP/URL where this server is visibly accessible from.
	///
	/// e.g. 'www.my_winvoice.com', 'localhost', '123.4.18.1'
	#[arg(default_value_t = String::from("localhost"), long, short)]
	domain: String,

	/// The Winvoice adapter which will be used for this server.
	#[command(subcommand)]
	command: Command,

	/// The file containing the key to use for TLS. Must be in PEM format.
	#[arg(long, short, value_name = "FILE")]
	key: PathBuf,

	/// A **cryptographically random** key which will be used to sign refresh tokens. Must be at
	/// least 32-bytes.
	///
	/// If one is not provided, a random one will be generated.
	#[arg(long, short = 'S', value_name = "KEY")]
	refresh_secret: Option<Vec<u8>>,

	/// The maximum duration that a user may be logged in before requiring them to log in again.
	#[arg(
		default_value = "1month",
		long,
		short,
		value_name = "DURATION",
		value_parser = humantime::parse_duration,
	)]
	refresh_ttl: Duration,

	/// The amount of time that a [`Database`](sqlx::Database) connection may be held open by an
	/// user before it is closed.
	#[arg(
		default_value = "1ms",
		long,
		short = 'i',
		value_name = "DURATION",
		value_parser = humantime::parse_duration,
	)]
	session_idle: Duration,

	/// The amount of time that a session is valid for.
	#[arg(
		default_value = "5min",
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
		let refresh_secret =
			self.refresh_secret.map_or_else(Key::generate, |s| Key::derive_from(&s));
		let refresh_ttl = time::Duration::try_from(self.session_ttl)?;
		let session_ttl = time::Duration::try_from(self.session_ttl)?;
		let tls = RustlsConfig::from_pem_file(self.certificate, self.key).await?;

		match self.command
		{
			#[cfg(feature = "postgres")]
			Command::Postgres(p) => p.run(
				self.address,
				self.domain,
				refresh_secret,
				refresh_ttl,
				self.session_idle,
				session_ttl,
				self.timeout,
				tls,
			),
		}
		.await
	}
}
