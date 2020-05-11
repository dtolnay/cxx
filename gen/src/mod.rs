// Functionality that is shared between the cxx_build::bridge entry point and
// the cxxbridge CLI command.

mod error;
mod find;
pub(super) mod include;
pub(super) mod out;
mod write;

use self::error::{format_err, Error, Result};
use crate::syntax::namespace::Namespace;
use crate::syntax::report::Errors;
use crate::syntax::{self, check, Types};
use std::fs;
use std::path::Path;
use syn::Item;

struct Input {
    namespace: Namespace,
    module: Vec<Item>,
}

#[derive(Default)]
pub(super) struct Opt {
    /// Any additional headers to #include
    pub include: Vec<String>,
}

pub(super) fn do_generate_bridge(path: &Path, opt: Opt) -> Vec<u8> {
    let header = false;
    generate(path, opt, header)
}

pub(super) fn do_generate_header(path: &Path, opt: Opt) -> Vec<u8> {
    let header = true;
    generate(path, opt, header)
}

fn generate(path: &Path, opt: Opt, header: bool) -> Vec<u8> {
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(err) => format_err(path, "", Error::Io(err)),
    };
    match (|| -> Result<_> {
        proc_macro2::fallback::force();
        let ref mut errors = Errors::new();
        let syntax = syn::parse_file(&source)?;
        let bridge = find::find_bridge_mod(syntax)?;
        let ref namespace = bridge.namespace;
        let ref apis = syntax::parse_items(errors, bridge.module);
        let ref types = Types::collect(errors, apis);
        errors.propagate()?;
        check::typecheck(errors, namespace, apis, types);
        errors.propagate()?;
        let out = write::gen(namespace, apis, types, opt, header);
        Ok(out)
    })() {
        Ok(out) => out.content(),
        Err(err) => format_err(path, &source, err),
    }
}
