use clap::Subcommand;

#[cfg(feature = "postgres")]
use super::postgres::Postgres;
use crate::DynResult;

/// The specific command that CLInvoice should run.
#[derive(Clone, Debug, Subcommand)]
pub enum Command
{
	#[allow(missing_docs)]
	#[cfg(feature = "postgres")]
	Postgres(Postgres),
}
