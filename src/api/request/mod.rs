//! This module contains all of the valid [HTTP](axum::http) requests that the server may
//! [respond](super::response) to.

mod delete;
mod get;
mod post;

/// The request to [update](winvoice_adapter::Updatable::update) some information.
pub type Patch<T> = Delete<T>;
pub use delete::Delete;
pub use get::Get;
pub use post::Post;
