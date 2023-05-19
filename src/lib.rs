//! Using `winvoice-server` as a dependency will allow you to access strongly-typed versions of its
//! API. Be sure to use `default-features = false`, or else it will pull in [`axum`], [`clap`],
//! etc.

pub mod api;
