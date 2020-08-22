//! The CXX code generator for constructing and compiling C++ code.
//!
//! This is intended to be embedded into higher-level code generators.

mod gen;
mod syntax;

use crate::gen::Opt;
use proc_macro2::TokenStream;

pub use crate::gen::{Error, GeneratedCode, Result};

/// Generate C++ bindings code from a Rust token stream. This should be a Rust
/// token stream which somewhere contains a `#[cxx::bridge] mod {}`.
pub fn generate_header_and_cc(rust_source: TokenStream) -> Result<GeneratedCode> {
    gen::do_generate_from_tokens(rust_source, Opt::default())
}

#[cfg(test)]
mod test {
    use quote::quote;

    #[test]
    fn test_positive() {
        let rs = quote! {
            #[cxx::bridge]
            mod ffi {
                extern "C" {
                    fn in_C();
                }
                extern "Rust" {
                    fn in_rs();
                }
            }
        };
        let code = crate::generate_header_and_cc(rs).unwrap();
        assert!(code.cxx.len() > 0);
        assert!(code.header.len() > 0);
    }

    #[test]
    fn test_negative() {
        let rs = quote! {};
        assert!(crate::generate_header_and_cc(rs).is_err())
    }
}
