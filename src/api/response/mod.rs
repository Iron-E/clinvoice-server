//! This module contains all of the responses to an HTTP [request](super::request) which the
//! [`winvoice_server`](crate) may issue.

mod delete;
mod export;
mod get;
mod login;
mod logout;
mod post;
mod version;
mod who_am_i;

/// The response to [updating](winvoice_adapter::Updatable::update) some information.
#[allow(dead_code)]
pub type Patch = Delete;
pub use delete::Delete;
pub use export::Export;
pub use get::Get;
pub use login::Login;
pub use logout::Logout;
pub use post::Post;
pub use version::Version;
pub use who_am_i::WhoAmI;
