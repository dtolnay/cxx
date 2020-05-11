use crate::gen::{Error, Input, Result};
use crate::syntax::namespace::Namespace;
use quote::quote;
use syn::{Attribute, File, Item};

pub(super) fn find_bridge_mod(syntax: File) -> Result<Input> {
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
