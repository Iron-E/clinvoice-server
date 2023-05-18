mod command;
#[cfg(feature = "postgres")]
mod postgres;

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
	/// The specific CLInvoice subcommand to run.
	#[command(subcommand)]
	command: Command,
}

impl Args
{
	pub async fn run(self) -> DynResult<()>
	{
		self.command.run().await
	}
}
