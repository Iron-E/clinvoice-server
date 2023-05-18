mod command;
#[cfg(feature = "postgres")]
mod postgres;

use std::{net::SocketAddr, path::PathBuf};

use axum_server::tls_rustls::RustlsConfig;
use clap::Parser;
use command::Command;

use crate::DynResult;

/// CLInvoice is a tool to track and generate invoices from the command line. Pass --help for more.
///
/// It is capable of managing information about clients, employees, jobs, timesheets, and exporting
/// the information into the format of your choice.
#[derive(Clone, Debug, Parser)]
#[command(version = "0.1.0-alpha.1")]
pub struct Args
{
	/// The IP address to bind the CLInvoice server to.
	#[arg(default_value = "127.0.0.1:3000", long, short)]
	address: SocketAddr,

	/// The file containing the certificate to use for TLS. Must be in PEM format.
	#[arg(long, short)]
	certificate: PathBuf,

	/// The CLInvoice adapter which will be used for this server.
	#[command(subcommand)]
	command: Command,

	/// The file containing the key to use for TLS. Must be in PEM format.
	#[arg(long, short)]
	key: PathBuf,
}

impl Args
{
	pub async fn run(self) -> DynResult<()>
	{
		let tls = RustlsConfig::from_pem_file(self.certificate, self.key).await?;
		match self.command
		{
			Command::Postgres(p) => p.run(self.address, tls),
		}
		.await
	}
}
