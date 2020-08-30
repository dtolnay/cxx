//! The CXX code generator for constructing and compiling C++ code.
//!
//! This is intended to be embedded into higher-level code generators.

#![allow(dead_code)]

mod gen;
mod syntax;

pub use crate::gen::Opt;
use proc_macro2::TokenStream;
use std::error::Error as StdError;
use std::fmt::{self, Debug, Display};

/// Results of code generation.
pub struct GeneratedCode {
    /// The bytes of a C++ header file.
    pub header: Vec<u8>,
    /// The bytes of a C++ implementation file (e.g. .cc, cpp etc.)
    pub cxx: Vec<u8>,
}

pub struct Error(crate::gen::Error);

/// Generate C++ bindings code from a Rust token stream. This should be a Rust
/// token stream which somewhere contains a `#[cxx::bridge] mod {}`.
pub fn generate_header_and_cc(rust_source: TokenStream, opt: &Opt) -> Result<GeneratedCode, Error> {
    let syntax = syn::parse2(rust_source)
        .map_err(crate::gen::Error::from)
        .map_err(Error)?;
    match gen::generate(syntax, opt, true, true).map_err(Error)? {
        (Some(header), Some(cxx)) => Ok(GeneratedCode { header, cxx }),
        _ => panic!("Unexpected generation"),
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.0.source()
    }
}
