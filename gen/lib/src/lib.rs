//! The CXX code generator for constructing and compiling C++ code.
//!
//! This is intended as a mechanism for embedding the `cxx` crate into
//! higher-level code generators. See [dtolnay/cxx#235] and
//! [https://github.com/google/autocxx].
//!
//! [dtolnay/cxx#235]: https://github.com/dtolnay/cxx/issues/235
//! [https://github.com/google/autocxx]: https://github.com/google/autocxx

#![allow(dead_code)]
#![allow(
    clippy::inherent_to_string,
    clippy::new_without_default,
    clippy::or_fun_call,
    clippy::toplevel_ref_arg
)]

mod error;
mod gen;
mod syntax;

pub use crate::error::Error;
pub use crate::gen::{GeneratedCode, Opt};
use proc_macro2::TokenStream;

const CXX_HEADER: &'static str = include_str!("../../../include/cxx.h");

/// Generate C++ bindings code from a Rust token stream. This should be a Rust
/// token stream which somewhere contains a `#[cxx::bridge] mod {}`.
pub fn generate_header_and_cc(rust_source: TokenStream, opt: &Opt) -> Result<GeneratedCode, Error> {
    let syntax = syn::parse2(rust_source)
        .map_err(crate::gen::Error::from)
        .map_err(Error::from)?;
    gen::generate(syntax, opt).map_err(Error::from)
}

/// Returns the complete contents of the cxx.h header.
pub fn get_cxx_header_contents() -> &'static str {
    CXX_HEADER
}
