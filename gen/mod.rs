// Functionality that is shared between the cxx::generate_bridge entry point and
// the cmd.

mod error;
pub(super) mod include;
pub(super) mod out;
mod write;

use self::error::format_err;
use crate::syntax::namespace::Namespace;
use crate::syntax::{self, check, Types};
use quote::quote;
use std::fs;
use std::io;
use std::path::Path;
use syn::{Attribute, File, Item};
use thiserror::Error;

pub(super) type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Error, Debug)]
pub(super) enum Error {
    #[error("no #[cxx::bridge] module found")]
    NoBridgeMod,
    #[error("#[cxx::bridge] module must have inline contents")]
    OutOfLineMod,
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Syn(#[from] syn::Error),
}

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
        let syntax = syn::parse_file(&source)?;
        let bridge = find_bridge_mod(syntax)?;
        let apis = syntax::parse_items(bridge.module)?;
        let types = Types::collect(&apis)?;
        check::typecheck(&apis, &types)?;
        let out = write::gen(bridge.namespace, &apis, &types, opt, header);
        Ok(out)
    })() {
        Ok(out) => out.content(),
        Err(err) => format_err(path, &source, err),
    }
}

fn find_bridge_mod(syntax: File) -> Result<Input> {
    for item in syntax.items {
        if let Item::Mod(item) = item {
            for attr in &item.attrs {
                let path = &attr.path;
                if quote!(#path).to_string() == "cxx :: bridge" {
                    let module = match item.content {
                        Some(module) => module.1,
                        None => {
                            return Err(Error::Syn(syn::Error::new_spanned(
                                item,
                                Error::OutOfLineMod,
                            )));
                        }
                    };
                    let namespace = parse_args(attr)?;
                    return Ok(Input { namespace, module });
                }
            }
        }
    }
    Err(Error::NoBridgeMod)
}

fn parse_args(attr: &Attribute) -> syn::Result<Namespace> {
    if attr.tokens.is_empty() {
        Ok(Namespace::none())
    } else {
        attr.parse_args()
    }
}
