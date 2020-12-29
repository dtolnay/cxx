#![allow(
    clippy::cast_sign_loss,
    clippy::default_trait_access,
    clippy::doc_markdown,
    clippy::enum_glob_use,
    clippy::filter_map,
    clippy::inherent_to_string,
    clippy::items_after_statements,
    clippy::large_enum_variant,
    clippy::match_bool,
    clippy::match_same_arms,
    clippy::module_name_repetitions,
    clippy::needless_pass_by_value,
    clippy::new_without_default,
    clippy::nonminimal_bool,
    clippy::option_if_let_else,
    clippy::or_fun_call,
    clippy::redundant_else,
    clippy::shadow_unrelated,
    clippy::similar_names,
    clippy::single_match,
    clippy::single_match_else,
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::toplevel_ref_arg,
    clippy::useless_let_if_seq
)]

extern crate proc_macro;

mod derive;
mod expand;
mod syntax;
mod type_id;

use crate::syntax::file::Module;
use crate::syntax::namespace::Namespace;
use crate::syntax::qualified::QualifiedName;
use proc_macro::TokenStream;
use syn::parse::{Parse, ParseStream, Parser, Result};
use syn::parse_macro_input;

/// `#[cxx::bridge] mod ffi { ... }`
///
/// Refer to the crate-level documentation for the explanation of how this macro
/// is intended to be used.
///
/// The only additional thing to note here is namespace support &mdash; if the
/// types and functions on the `extern "C++"` side of our bridge are in a
/// namespace, specify that namespace as an argument of the cxx::bridge
/// attribute macro.
///
/// ```
/// #[cxx::bridge(namespace = "mycompany::rust")]
/// # mod ffi {}
/// ```
///
/// The types and functions from the `extern "Rust"` side of the bridge will be
/// placed into that same namespace in the generated C++ code.
#[proc_macro_attribute]
pub fn bridge(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = syntax::error::ERRORS;

    let namespace = match Namespace::parse_bridge_attr_namespace.parse(args) {
        Ok(namespace) => namespace,
        Err(err) => return err.to_compile_error().into(),
    };
    let mut ffi = parse_macro_input!(input as Module);
    ffi.namespace = namespace;

    expand::bridge(ffi)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro]
pub fn type_id(input: TokenStream) -> TokenStream {
    struct TypeId(QualifiedName);

    impl Parse for TypeId {
        fn parse(input: ParseStream) -> Result<Self> {
            QualifiedName::parse_quoted_or_unquoted(input).map(TypeId)
        }
    }

    let arg = parse_macro_input!(input as TypeId);
    type_id::expand(arg.0).into()
}
