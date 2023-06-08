//! This module contains strongly-typed versions of all JSON information sent via the
//! [`winvoice_server`].

pub mod r#match;
pub mod request;
pub mod response;
pub mod schema;
mod status;

use std::sync::OnceLock;

use semver::{BuildMetadata, Prerelease, Version};
pub use status::{Code, Status};

/// The header which is used to advertise the semantic version that the client accepts.
pub const HEADER: &str = "Api-Version";

/// The current API version.
static VERSION: OnceLock<Version> = OnceLock::new();

// static VERSION: Version = Version {
// 	build: BuildMetadata::EMPTY,
// 	major: 0,
// 	minor: 1,
// 	patch: 0,
// 	pre: Prerelease::new("alpha.1").unwrap(),
// };

/// The current API version.
pub fn version() -> &'static Version
{
	VERSION.get_or_init(|| Version {
		build: BuildMetadata::EMPTY,
		major: 0,
		minor: 1,
		patch: 0,
		pre: Prerelease::new("alpha.1").unwrap(),
	})
}
