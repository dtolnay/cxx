//! The CXX code generator for constructing and compiling C++ code.
//!
//! This is intended to be embedded into higher-level code generators.

mod gen;
mod syntax;

pub use crate::gen::Opt;
use proc_macro2::TokenStream;

pub use crate::gen::{Error, Result};

/// Results of code generation.
pub struct GeneratedCode {
    /// The bytes of a C++ header file.
    pub header: Vec<u8>,
    /// The bytes of a C++ implementation file (e.g. .cc, cpp etc.)
    pub cxx: Vec<u8>,
}

/// Generate C++ bindings code from a Rust token stream. This should be a Rust
/// token stream which somewhere contains a `#[cxx::bridge] mod {}`.
pub fn generate_header_and_cc(rust_source: TokenStream, opt: Opt) -> Result<GeneratedCode> {
    let syntax = syn::parse2(rust_source)?;
    match gen::generate(syntax, opt, true, true) {
        Ok((Some(header), Some(cxx))) => Ok(GeneratedCode { header, cxx }),
        Err(err) => Err(err),
        _ => panic!("Unexpected generation"),
    }
}
