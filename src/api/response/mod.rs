//! This module contains all of the responses to an HTTP [request](super::request) which the
//! [`winvoice_server`] may issue.

mod login;
mod logout;

pub use login::Login;
pub use logout::Logout;
