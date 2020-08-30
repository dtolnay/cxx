// Functionality that is shared between the cxx_build::bridge entry point and
// the cxxbridge CLI command.

mod error;
mod file;
pub(super) mod include;
pub(super) mod out;
mod write;

#[cfg(test)]
mod tests;

use self::error::format_err;
pub use self::error::{Error, Result};
use self::file::File;
use crate::syntax::report::Errors;
use crate::syntax::{self, check, Types};
use proc_macro2::TokenStream;
use std::clone::Clone;
use std::fs;
use std::path::Path;

/// Options for C++ code generation.
#[derive(Default, Clone)]
pub struct Opt {
    /// Any additional headers to #include
    pub include: Vec<String>,
    /// Whether to set __attribute__((visibility("default")))
    /// or similar annotations on function implementations.
    pub cxx_impl_annotations: Option<String>,
}

/// Results of code generation.
pub struct GeneratedCode {
    /// The bytes of a C++ header file.
    pub header: Vec<u8>,
    /// The bytes of a C++ implementation file (e.g. .cc, cpp etc.)
    pub cxx: Vec<u8>,
}

pub(super) fn do_generate_bridge(path: &Path, opt: Opt) -> Vec<u8> {
    let header = false;
    generate_from_path(path, opt, header)
}

pub(super) fn do_generate_header(path: &Path, opt: Opt) -> Vec<u8> {
    let header = true;
    generate_from_path(path, opt, header)
}

pub(super) fn do_generate_from_tokens(
    tokens: TokenStream,
    opt: Opt,
) -> std::result::Result<GeneratedCode, Error> {
    let syntax = syn::parse2::<File>(tokens)?;
    match generate(syntax, opt, true, true) {
        Ok((Some(header), Some(cxx))) => Ok(GeneratedCode { header, cxx }),
        Err(err) => Err(err),
        _ => panic!("Unexpected generation"),
    }
}

fn generate_from_path(path: &Path, opt: Opt, header: bool) -> Vec<u8> {
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(err) => format_err(path, "", Error::Io(err)),
    };
    match generate_from_string(&source, opt, header) {
        Ok(out) => out,
        Err(err) => format_err(path, &source, err),
    }
}

fn generate_from_string(source: &str, opt: Opt, header: bool) -> Result<Vec<u8>> {
    let mut source = source;
    if source.starts_with("#!") && !source.starts_with("#![") {
        let shebang_end = source.find('\n').unwrap_or(source.len());
        source = &source[shebang_end..];
    }
    let syntax: File = syn::parse_str(source)?;
    let results = generate(syntax, opt, header, !header)?;
    match results {
        (Some(hdr), None) => Ok(hdr),
        (None, Some(cxx)) => Ok(cxx),
        _ => panic!("Unexpected generation"),
    }
}

fn generate(
    syntax: File,
    opt: Opt,
    gen_header: bool,
    gen_cxx: bool,
) -> Result<(Option<Vec<u8>>, Option<Vec<u8>>)> {
    proc_macro2::fallback::force();
    let ref mut errors = Errors::new();
    let bridge = syntax
        .modules
        .into_iter()
        .next()
        .ok_or(Error::NoBridgeMod)?;
    let ref namespace = bridge.namespace;
    let trusted = bridge.unsafety.is_some();
    let ref apis = syntax::parse_items(errors, bridge.content, trusted);
    let ref types = Types::collect(errors, apis);
    errors.propagate()?;
    check::typecheck(errors, namespace, apis, types);
    errors.propagate()?;
    // Some callers may wish to generate both header and C++
    // from the same token stream to avoid parsing twice. But others
    // only need to generate one or the other.
    let hdr = if gen_header {
        Some(write::gen(namespace, apis, types, opt.clone(), true).content())
    } else {
        None
    };
    let cxx = if gen_cxx {
        Some(write::gen(namespace, apis, types, opt, false).content())
    } else {
        None
    };
    Ok((hdr, cxx))
}
