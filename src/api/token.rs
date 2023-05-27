//! This module contains data types which are used by [`winvoice_server`] to refer to a
//! unique user identity.

/// A refresh token.
pub type Token = [u8; 5];
