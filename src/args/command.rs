use clap::Subcommand;

#[cfg(feature = "postgres")]
use super::postgres::Postgres;
use crate::{DynResult, Run};

/// The specific command that CLInvoice should run.
#[derive(Clone, Debug, Subcommand)]
pub enum Command
{
	#[allow(missing_docs)]
	#[cfg(feature = "postgres")]
	Postgres(Postgres),
}

#[async_trait::async_trait]
impl Run for Command
{
	async fn run(self) -> DynResult<()>
	{
		match self
		{
			Self::Postgres(p) => p.run().await?,
		};

		Ok(())
	}
}
