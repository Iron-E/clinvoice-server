//! Using this crate as a dependency will allow you to access strongly-typed versions of its
//! [API](api). Be sure to use `default-features = false`, or else it will pull in [`axum`],
//! [`clap`], etc.

pub mod api;
mod bool_ext;
mod dyn_result;
pub mod r#match;
pub mod permissions;
mod result_ext;
pub mod schema;
mod utils;

pub use bool_ext::BoolExt;
pub use result_ext::ResultExt;
