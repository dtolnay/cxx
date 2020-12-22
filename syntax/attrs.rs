use crate::syntax::namespace::Namespace;
use crate::syntax::report::Errors;
use crate::syntax::Atom::{self, *};
use crate::syntax::{Derive, Doc};
use proc_macro2::Ident;
use syn::parse::{ParseStream, Parser as _};
use syn::{Attribute, Error, LitStr, Path, Result, Token};

// Intended usage:
//
//     let mut doc = Doc::new();
//     let mut cxx_name = None;
//     let mut rust_name = None;
//     /* ... */
//     attrs::parse(
//         cx,
//         &item.attrs,
//         attrs::Parser {
//             doc: Some(&mut doc),
//             cxx_name: Some(&mut cxx_name),
//             rust_name: Some(&mut rust_name),
//             /* ... */
//             ..Default::default()
//         },
//     );
//
#[derive(Default)]
pub struct Parser<'a> {
    pub doc: Option<&'a mut Doc>,
    pub derives: Option<&'a mut Vec<Derive>>,
    pub repr: Option<&'a mut Option<Atom>>,
    pub namespace: Option<&'a mut Namespace>,
    pub cxx_name: Option<&'a mut Option<Ident>>,
    pub rust_name: Option<&'a mut Option<Ident>>,

    // Suppress clippy needless_update lint ("struct update has no effect, all
    // the fields in the struct have already been specified") when preemptively
    // writing `..Default::default()`.
    pub(crate) _more: (),
}

pub(super) fn parse(cx: &mut Errors, attrs: &[Attribute], mut parser: Parser) {
    for attr in attrs {
        if attr.path.is_ident("doc") {
            match parse_doc_attribute.parse2(attr.tokens.clone()) {
                Ok(lit) => {
                    if let Some(doc) = &mut parser.doc {
                        doc.push(lit);
                        continue;
                    }
                }
                Err(err) => return cx.push(err),
            }
        } else if attr.path.is_ident("derive") {
            match attr.parse_args_with(|attr: ParseStream| parse_derive_attribute(cx, attr)) {
                Ok(attr) => {
                    if let Some(derives) = &mut parser.derives {
                        derives.extend(attr);
                        continue;
                    }
                }
                Err(err) => return cx.push(err),
            }
        } else if attr.path.is_ident("repr") {
            match attr.parse_args_with(parse_repr_attribute) {
                Ok(attr) => {
                    if let Some(repr) = &mut parser.repr {
                        **repr = Some(attr);
                        continue;
                    }
                }
                Err(err) => return cx.push(err),
            }
        } else if attr.path.is_ident("namespace") {
            match parse_namespace_attribute.parse2(attr.tokens.clone()) {
                Ok(attr) => {
                    if let Some(namespace) = &mut parser.namespace {
                        **namespace = attr;
                        continue;
                    }
                }
                Err(err) => return cx.push(err),
            }
        } else if attr.path.is_ident("cxx_name") {
            match parse_function_alias_attribute.parse2(attr.tokens.clone()) {
                Ok(attr) => {
                    if let Some(cxx_name) = &mut parser.cxx_name {
                        **cxx_name = Some(attr);
                        continue;
                    }
                }
                Err(err) => return cx.push(err),
            }
        } else if attr.path.is_ident("rust_name") {
            match parse_function_alias_attribute.parse2(attr.tokens.clone()) {
                Ok(attr) => {
                    if let Some(rust_name) = &mut parser.rust_name {
                        **rust_name = Some(attr);
                        continue;
                    }
                }
                Err(err) => return cx.push(err),
            }
        }
        return cx.error(attr, "unsupported attribute");
    }
}

fn parse_doc_attribute(input: ParseStream) -> Result<LitStr> {
    input.parse::<Token![=]>()?;
    let lit: LitStr = input.parse()?;
    Ok(lit)
}

fn parse_derive_attribute(cx: &mut Errors, input: ParseStream) -> Result<Vec<Derive>> {
    let paths = input.parse_terminated::<Path, Token![,]>(Path::parse_mod_style)?;

    let mut derives = Vec::new();
    for path in paths {
        if let Some(ident) = path.get_ident() {
            if let Some(derive) = Derive::from(ident) {
                derives.push(derive);
                continue;
            }
        }
        cx.error(path, "unsupported derive");
    }
    Ok(derives)
}

fn parse_repr_attribute(input: ParseStream) -> Result<Atom> {
    let begin = input.cursor();
    let ident: Ident = input.parse()?;
    if let Some(atom) = Atom::from(&ident) {
        match atom {
            U8 | U16 | U32 | U64 | Usize | I8 | I16 | I32 | I64 | Isize if input.is_empty() => {
                return Ok(atom);
            }
            _ => {}
        }
    }
    Err(Error::new_spanned(
        begin.token_stream(),
        "unrecognized repr",
    ))
}

fn parse_namespace_attribute(input: ParseStream) -> Result<Namespace> {
    input.parse::<Token![=]>()?;
    let namespace = input.parse::<Namespace>()?;
    Ok(namespace)
}

fn parse_function_alias_attribute(input: ParseStream) -> Result<Ident> {
    input.parse::<Token![=]>()?;
    if input.peek(LitStr) {
        let lit: LitStr = input.parse()?;
        lit.parse()
    } else {
        input.parse()
    }
}
