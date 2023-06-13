//! Using this crate as a dependency will allow you to access strongly-typed versions of its
//! [API](api). Be sure to use `default-features = false`, or else it will pull in [`axum`],
//! [`clap`], etc.

pub mod api;
mod dyn_result;
pub mod r#match;
pub mod permissions;
pub mod schema;
mod utils;
