//! This module contains all of the valid [HTTP](axum::http) requests that the server may
//! [respond](super::response) to.

mod retrieve;

pub use retrieve::Retrieve;
