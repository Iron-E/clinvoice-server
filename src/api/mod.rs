//! This module contains strongly-typed versions of all JSON information sent via the
//! [`winvoice-server`](crate).

pub mod request;
pub mod response;
mod status;

pub use status::{Code as StatusCode, Status};
