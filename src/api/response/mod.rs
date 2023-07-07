//! This module contains all of the responses to an HTTP [request](super::request) which the
//! [`winvoice_server`](crate) may issue.

mod get;
mod login;
mod logout;
mod version;

pub use get::Get;
pub use login::Login;
pub use logout::Logout;
pub use version::Version;
