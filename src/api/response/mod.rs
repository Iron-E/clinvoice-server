//! This module contains all of the responses to an HTTP [request](super::request) which the
//! [`winvoice_server`](crate) may issue.
//!
//! Responses will typically contain a JSON body which can deserialize into one of the types specified in this module.
//! (i.e. a `DELETE` request will typically respond with [`Delete`], a login request will respond with valid [`Login`]
//! JSON, etc.)
//!
//! Alternatively the server may respond with [`Version`] to any request which is made on a client that specifies an API
//! version outside the [API version](super::version) being run by the server.
//!
//! If an error is encountered while attempting to service a request, this JSON may be absent. Such scenarios include:
//!
//! * The user is not logged in (for all [routes](super::routes) except [`/login`](super::routes::LOGIN)) (code 401);

mod delete;
mod export;
mod login;
mod logout;
mod post;
mod put;
mod version;
mod who_am_i;

/// The response to [updating](winvoice_adapter::Updatable::update) some information.
#[allow(dead_code)]
pub type Patch = Delete;
pub use delete::Delete;
pub use export::Export;
pub use login::Login;
pub use logout::Logout;
pub use post::Post;
pub use put::Put;
pub use version::Version;
pub use who_am_i::WhoAmI;
