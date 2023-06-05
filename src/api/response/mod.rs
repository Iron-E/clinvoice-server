//! This module contains all of the responses to an HTTP [request](super::request) which the
//! [`winvoice_server`] may issue.

mod login;
mod logout;
mod retrieve;

pub use login::Login;
pub use logout::Logout;
pub use retrieve::Retrieve;
