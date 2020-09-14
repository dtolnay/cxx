use crate::syntax::qualified::QualifiedName;
use crate::syntax::report::Errors;
use crate::syntax::Atom::{self, *};
use crate::syntax::{Derive, Doc, Namespace};
use proc_macro2::Ident;
use std::collections::HashMap;
use syn::parse::{ParseStream, Parser as _};
use syn::{Attribute, Error, LitStr, Path, Result, Token};

#[derive(Default)]
pub struct Parser<'a> {
    pub doc: Option<&'a mut Doc>,
    pub derives: Option<&'a mut Vec<Derive>>,
    pub repr: Option<&'a mut Option<Atom>>,
    pub alias_namespaces: Option<&'a mut HashMap<QualifiedName, Namespace>>,
    pub ignore_unsupported: bool,
}

pub(super) fn parse_doc(cx: &mut Errors, attrs: &[Attribute]) -> Doc {
    let mut doc = Doc::new();
    parse(
        cx,
        attrs,
        Parser {
            doc: Some(&mut doc),
            ..Parser::default()
        },
    );
    doc
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
            match attr.parse_args_with(parse_derive_attribute) {
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
        } else if is_cxx_alias_namespace_attr(attr) {
            match attr.parse_args_with(parse_namespace_attribute) {
                Ok((name, namespace)) => {
                    if let Some(map) = &mut parser.alias_namespaces {
                        if let Some(existing) = map.get(&name) {
                            return cx.error(
                                attr,
                                format!(
                                    "conflicting cxx::alias_namespace attributes for {}: {}, {}",
                                    name, existing, namespace
                                ),
                            );
                        }
                        map.insert(name, namespace);
                        continue;
                    }
                }
                Err(err) => return cx.push(err),
            }
        }
        if !parser.ignore_unsupported {
            return cx.error(attr, "unsupported attribute");
        }
    }
}

fn is_cxx_alias_namespace_attr(attr: &Attribute) -> bool {
    let path = &attr.path.segments;
    path.len() == 2 && path[0].ident == "cxx" && path[1].ident == "alias_namespace"
}

fn parse_doc_attribute(input: ParseStream) -> Result<LitStr> {
    input.parse::<Token![=]>()?;
    let lit: LitStr = input.parse()?;
    Ok(lit)
}

fn parse_derive_attribute(input: ParseStream) -> Result<Vec<Derive>> {
    input
        .parse_terminated::<Path, Token![,]>(Path::parse_mod_style)?
        .into_iter()
        .map(|path| {
            if let Some(ident) = path.get_ident() {
                if let Some(derive) = Derive::from(ident) {
                    return Ok(derive);
                }
            }
            Err(Error::new_spanned(path, "unsupported derive"))
        })
        .collect()
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

fn parse_namespace_attribute(input: ParseStream) -> Result<(QualifiedName, Namespace)> {
    let name = QualifiedName::parse_quoted_or_unquoted(input)?;
    input.parse::<Token![=]>()?;
    let namespace = input.parse::<Namespace>()?;
    Ok((name, namespace))
}
