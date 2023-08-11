//! `winvoice-server` is WIP backend for Winvoice libraries. It aims to allow any
//! number of different frontends, such as [winvoice-cli](https://github.com/Iron-E/winvoice-cli) or
//! [winvoice-gui](https://github.com/Iron-E/winvoice-gui), to communicate with it without having to be
//! written in Rust or re-implement common procedures.
//!
//! # Usage
//!
//! * For basic information, run `winvoice-server help` from the command line.
//! * For an in-depth guide, see the [wiki](https://github.com/Iron-E/winvoice-server/wiki/Usage).
//!
//! ## Installation
//!
//! Requirements:
//!
//! * [`cargo`](https://github.com/rust-lang/cargo)
//!
//! ```sh
//! cargo install \
//!   --features <adapters> \
//!   --git https://github.com/Iron-E/winvoice-server \
//!   --root=<desired install folder>
//! ```
//!
//! ## API
//!
//! You can add `winvoice-server` to your `[dependencies]` to access the `winvoice_server::api`
//! directly:
//!
//! ```toml
//! [dependencies.winvoice-server]
//! branch = "master"
//! default-features = false
//! git = "https://github.com/Iron-E/winvoice-server"
//! ```
//!
//! If you are working with another language, see [the docs](TODO).
//!
//! # Development
//!
//! ## Self-signed certificates
//!
//! I recommend the use of the tool [`mkcert`](https://github.com/FiloSottile/mkcert) to generate trusted certificates
//! on your local machine, for the purposes of writing a front-end.

#![allow(clippy::drop_non_drop, clippy::inconsistent_digit_grouping)]
#![forbid(unsafe_code)]
#![warn(
	missing_docs,
	clippy::alloc_instead_of_core,
	clippy::allow_attributes_without_reason,
	clippy::as_underscore,
	clippy::branches_sharing_code,
	clippy::cast_lossless,
	clippy::checked_conversions,
	clippy::cloned_instead_of_copied,
	clippy::dbg_macro,
	clippy::debug_assert_with_mut_call,
	clippy::doc_link_with_quotes,
	clippy::doc_markdown,
	clippy::empty_line_after_outer_attr,
	clippy::empty_structs_with_brackets,
	clippy::enum_glob_use,
	clippy::equatable_if_let,
	clippy::exit,
	clippy::explicit_into_iter_loop,
	clippy::explicit_iter_loop,
	clippy::fallible_impl_from,
	clippy::filetype_is_file,
	clippy::filter_map_next,
	clippy::flat_map_option,
	clippy::fn_to_numeric_cast_any,
	clippy::format_push_string,
	clippy::from_iter_instead_of_collect,
	clippy::get_unwrap,
	clippy::implicit_clone,
	clippy::inefficient_to_string,
	clippy::items_after_statements,
	clippy::manual_assert,
	clippy::manual_ok_or,
	clippy::map_unwrap_or,
	clippy::match_same_arms,
	clippy::missing_const_for_fn,
	clippy::missing_panics_doc,
	clippy::multiple_inherent_impl,
	clippy::mut_mut,
	clippy::needless_continue,
	clippy::option_if_let_else,
	clippy::option_option,
	clippy::range_minus_one,
	clippy::range_plus_one,
	clippy::redundant_closure_for_method_calls,
	clippy::redundant_else,
	clippy::ref_binding_to_reference,
	clippy::ref_option_ref,
	clippy::same_functions_in_if_condition,
	clippy::single_char_lifetime_names,
	clippy::std_instead_of_core,
	clippy::str_to_string,
	clippy::string_add,
	clippy::string_add_assign,
	clippy::string_to_string,
	clippy::try_err,
	clippy::unnecessary_join,
	clippy::unnecessary_wraps,
	clippy::use_self,
	clippy::used_underscore_binding,
	clippy::wildcard_imports
)]

mod api;
mod args;
mod bool_ext;
mod dyn_result;
mod lock;
mod r#match;
mod permissions;
mod result_ext;
mod schema;
mod server;
mod twin_result;
mod utils;

use args::Args;
use clap::Parser;
use dyn_result::DynResult;
use result_ext::ResultExt;

/// Interprets arguments `winvoice` (if any) and executes the implied instruction.
#[tokio::main]
async fn main()
{
	if let Err(e) = Args::parse().run().await
	{
		eprintln!("{e}");

		#[cfg(debug_assertions)]
		eprintln!("Raw error: {e:#?}");
	}
}
