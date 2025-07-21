//! Test helpers for `syntax`-module-level parsing and checking of `#[cxx::bridge]`.

use crate::syntax::check::{self};
use crate::syntax::file::Module;
use crate::syntax::parse::parse_items;
use crate::syntax::report::Errors;
use crate::syntax::{Api, Types};

use proc_macro2::TokenStream;

/// Parses a `TokenStream` containing `#[cxx::bridge] mod { ... }`.
pub fn parse_apis(cxx_bridge: TokenStream) -> syn::Result<Vec<Api>> {
    let mut module = syn::parse2::<Module>(cxx_bridge)?;
    let mut errors = Errors::new();
    let apis = parse_items(&mut errors, &mut module);
    errors.propagate()?;

    Ok(apis)
}

/// Collects and type-checks types used in `apis`.
pub fn collect_types(apis: &[Api]) -> syn::Result<Types> {
    let mut errors = Errors::new();
    let types = Types::collect(&mut errors, apis);
    errors.propagate()?;

    let generator = check::Generator::Build;
    check::typecheck(&mut errors, apis, &types, generator);
    errors.propagate()?;

    Ok(types)
}
