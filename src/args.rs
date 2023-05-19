mod command;
#[cfg(feature = "postgres")]
mod postgres;

use core::time::Duration;
use std::{net::SocketAddr, path::PathBuf};

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

	/// The Winvoice adapter which will be used for this server.
	#[command(subcommand)]
	command: Command,

	/// The file containing the key to use for TLS. Must be in PEM format.
	#[arg(long, short, value_name = "FILE")]
	key: PathBuf,

	/// The maximum duration to run commands server before timing out (e.g. "5s", "15min").
	///
	/// When this argument is passed without a value, (e.g. `--timeout`), a timeout of 30 seconds
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
		let tls = RustlsConfig::from_pem_file(self.certificate, self.key).await?;
		match self.command
		{
			Command::Postgres(p) => p.run(self.address, tls, self.timeout),
		}
		.await
	}
}
