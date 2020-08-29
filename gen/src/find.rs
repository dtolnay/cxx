use crate::gen::{Error, File, Input, Result};
use crate::syntax::namespace::Namespace;
use syn::{Attribute, Item};

pub(super) fn find_bridge_mod(syntax: File) -> Result<Input> {
    match scan(syntax.items)? {
        Some(input) => Ok(input),
        None => Err(Error::NoBridgeMod),
    }
}

fn scan(items: Vec<Item>) -> Result<Option<Input>> {
    for item in items {
        if let Item::Mod(item) = item {
            for attr in &item.attrs {
                let path = &attr.path.segments;
                if path.len() == 2 && path[0].ident == "cxx" && path[1].ident == "bridge" {
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
                    return Ok(Some(Input { namespace, module }));
                }
            }
            if let Some(module) = item.content {
                if let Some(input) = scan(module.1)? {
                    return Ok(Some(input));
                }
            }
        }
    }
    Ok(None)
}

fn parse_args(attr: &Attribute) -> syn::Result<Namespace> {
    if attr.tokens.is_empty() {
        Ok(Namespace::none())
    } else {
        attr.parse_args()
    }
}
