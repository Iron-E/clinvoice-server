//! This module contains data types which are used by [`winvoice-server`](crate) to refer to a
//! unique user identity.

/// A refresh token.
pub type Token = [u8; 5];
