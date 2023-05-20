//! This module contains all of the responses to an HTTP [request](super::request) which the
//! [`winvoice-server`](crate) may issue.

mod login;

pub use login::Login;
