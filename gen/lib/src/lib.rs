//! The CXX code generator for constructing and compiling C++ code.
//!
//! This is intended as a mechanism for embedding the `cxx` crate into
//! higher-level code generators. See [dtolnay/cxx#235] and
//! [https://github.com/google/autocxx].
//!
//! [dtolnay/cxx#235]: https://github.com/dtolnay/cxx/issues/235
//! [https://github.com/google/autocxx]: https://github.com/google/autocxx

#![doc(html_root_url = "https://docs.rs/cxx-gen/0.7.194")]
#![deny(missing_docs)]
#![expect(dead_code)]
#![cfg_attr(not(check_cfg), allow(unexpected_cfgs))]
#![allow(
    clippy::cast_sign_loss,
    clippy::default_trait_access,
    clippy::elidable_lifetime_names,
    clippy::enum_glob_use,
    clippy::expl_impl_clone_on_copy, // https://github.com/rust-lang/rust-clippy/issues/15842
    clippy::inherent_to_string,
    clippy::items_after_statements,
    clippy::match_bool,
    clippy::match_like_matches_macro,
    clippy::match_same_arms,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::needless_continue,
    clippy::needless_lifetimes,
    clippy::needless_pass_by_value,
    clippy::nonminimal_bool,
    clippy::precedence,
    clippy::redundant_else,
    clippy::ref_option,
    clippy::similar_names,
    clippy::single_match_else,
    clippy::struct_excessive_bools,
    clippy::struct_field_names,
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::toplevel_ref_arg,
    clippy::uninlined_format_args
)]
#![allow(unknown_lints, mismatched_lifetime_syntaxes)]

mod error;
mod gen;
mod syntax;

pub use crate::error::Error;
pub use crate::gen::include::{Include, HEADER, IMPLEMENTATION};
pub use crate::gen::{CfgEvaluator, CfgResult, GeneratedCode, Opt};
pub use crate::syntax::IncludeKind;
use proc_macro2::TokenStream;

/// Generate C++ bindings code from a Rust token stream. This should be a Rust
/// token stream which somewhere contains a `#[cxx::bridge] mod {}`.
pub fn generate_header_and_cc(rust_source: TokenStream, opt: &Opt) -> Result<GeneratedCode, Error> {
    let syntax = syn::parse2(rust_source)
        .map_err(crate::gen::Error::from)
        .map_err(Error::from)?;
    gen::generate(syntax, opt).map_err(Error::from)
}

/// Format import symbols into OS-appropriate linker file content.
fn format_symbols_for_linker(symbols: &[String], target_os: &str) -> String {
    match target_os {
        "windows" => {
            let mut result = String::from("EXPORTS\n");
            for sym in symbols {
                result.push_str("    ");
                result.push_str(sym);
                result.push('\n');
            }
            result
        }
        "macos" => {
            let mut result = String::new();
            for sym in symbols {
                result.push_str("-U _");
                result.push_str(sym);
                result.push('\n');
            }
            result
        }
        _ => {
            // Linux and other Unix-like systems
            let mut result = String::from("{\n");
            for sym in symbols {
                result.push_str("  ");
                result.push_str(sym);
                result.push_str(";\n");
            }
            result.push_str("};");
            result
        }
    }
}

/// Format symbols that a shared library imports from an executable into the appropriate linker file format.
///
/// When a shared library calls functions defined in the executable that loads it, those symbols
/// must be declared as available to the linker. This function generates the platform-specific
/// files needed:
/// - **Windows**: `.def` file format (EXPORTS section) - used with `/DEF:` linker flag
/// - **macOS**: Linker arguments (`-U _symbol` for each) - marks symbols as dynamic_lookup
/// - **Linux**: Dynamic list format (`{ symbol; }`) - used with `--dynamic-list`
///
/// # Arguments
/// * `symbols` - The list of symbol names the library imports from the executable
/// * `target_os` - The target operating system ("windows", "macos", or "linux")
///
/// # Example
/// ```
/// let import_symbols = vec!["exe_callback".to_string(), "exe_get_constant".to_string()];
/// let content = cxx_gen::format_import_symbols_for_linker(&import_symbols, "linux");
/// // content will be "{\n  exe_callback;\n  exe_get_constant;\n};"
/// ```
pub fn format_import_symbols_for_linker(symbols: &[String], target_os: &str) -> String {
    format_symbols_for_linker(symbols, target_os)
}

/// Format symbols that a shared library exports into a Windows `.def` file format.
///
/// On Windows, shared libraries (DLLs) use `.def` files to explicitly list exported symbols.
/// This function generates the EXPORTS section needed for the library's `.def` file.
///
/// Note: This is Windows-specific. On Unix systems, exports are typically controlled via
/// version scripts (Linux) or visibility attributes, not separate export files.
///
/// # Arguments
/// * `symbols` - The list of symbol names the library exports
/// * `target_os` - Must be "windows"
///
/// # Panics
/// Panics if `target_os` is not "windows"
///
/// # Example
/// ```
/// let export_symbols = vec!["lib_process".to_string(), "lib_get_data".to_string()];
/// let content = cxx_gen::format_export_symbols_for_linker(&export_symbols, "windows");
/// // content will be "EXPORTS\n    lib_process\n    lib_get_data\n"
/// ```
pub fn format_export_symbols_for_linker(symbols: &[String], target_os: &str) -> String {
    if target_os != "windows" {
        panic!(
            "format_export_symbols_for_linker is only supported for Windows targets, got: {}",
            target_os
        );
    }
    format_symbols_for_linker(symbols, target_os)
}
