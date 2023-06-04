//! This module contains strongly-typed versions of all JSON information sent via the
//! [`winvoice_server`].

pub mod r#match;
pub mod request;
pub mod response;
pub mod schema;
mod status;

pub use status::{Code, Status};
