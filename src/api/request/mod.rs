//! This module contains all of the valid [HTTP](axum::http) requests that the server may
//! [respond](super::response) to.
//!
//! **All requests to the server** expect an [API version header](super::HEADER) to be present in order to ensure that
//! the client supports a [version range](https://docs.rs/semver/latest/semver/struct.VersionReq.html#method.parse)
//! which includes the server's [API version](super::version).
//!
//! **With the exception of** [`/login`](super::routes::LOGIN), [`/logout`](super::routes::LOGOUT),
//! [`/whoami`](super::routes::WHO_AM_I), **all requests** expect a JSON body, the schema of which depending on the
//! nature of the request. Requests to `DELETE` should use match a [`Delete`] request, `POST` to [`Post`], etc.

mod delete;
mod export;
mod post;
mod put;

/// The request to [update](winvoice_adapter::Updatable::update) some information.
pub type Patch<T> = Delete<T>;
pub use delete::Delete;
pub use export::Export;
pub use post::Post;
pub use put::Put;
