use crate::syntax::report::Errors;
use crate::syntax::Atom::{self, *};
use crate::syntax::{Derive, Doc};
use proc_macro2::Ident;
use syn::parse::{ParseStream, Parser as _};
use syn::{Attribute, Error, LitStr, Path, Result, Token};

#[derive(Default)]
pub struct Parser<'a> {
    pub doc: Option<&'a mut Doc>,
    pub derives: Option<&'a mut Vec<Derive>>,
    pub repr: Option<&'a mut Option<Atom>>,
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
        }
        return cx.error(attr, "unsupported attribute");
    }
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
