//! This module contains strongly-typed versions of all JSON information sent via the
//! [`winvoice_server`](crate).

mod status;

pub use status::{Code as StatusCode, Status};
