//! Contains a request to [retrieve](winvoice_adapter::Retrievable)

use serde::{Deserialize, Serialize};
use winvoice_schema::{Currency, Job};

/// The request to [delete](winvoice_adapter::Deletable::delete) some information.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Export
{
	/// The currency to export the [`Job`]s in. If [`None`], it will detect the
	/// [currency](winvoice_schema::Location::currency) for each [client](Organization).
	currency: Option<Currency>,

	/// See [`Job`]s to export.
	pub(crate) jobs: Vec<Job>,
}

impl Export
{
	/// Create a new [`Export`] request.
	#[allow(dead_code)]
	pub const fn new(currency: Option<Currency>, jobs: Vec<Job>) -> Self
	{
		Self { currency, jobs }
	}

	/// The [`Format`] that the [`jobs`](Export::jobs) will be exported to.
	#[allow(dead_code)]
	pub const fn currency(&self) -> Option<Currency>
	{
		self.currency
	}

	/// HACK: can't be an `Into` impl because rust-lang/rust#31844
	///
	/// # See also
	///
	/// * [`Retrieve::condition`]
	#[allow(clippy::missing_const_for_fn, dead_code)] // destructor cannot be evaluated at compile-time
	pub fn into_jobs(self) -> Vec<Job>
	{
		self.jobs
	}

	/// the [`jobs`] that will be [export](winvoice_export)ed.
	#[allow(dead_code)]
	pub fn jobs(&self) -> &[Job]
	{
		self.jobs.as_ref()
	}
}
