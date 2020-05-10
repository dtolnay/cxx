use crate::syntax::{Derive, Doc};
use proc_macro2::Ident;
use syn::parse::{ParseStream, Parser as _};
use syn::{Attribute, Error, LitStr, Path, Result, Token};

#[derive(Default)]
pub struct Parser<'a> {
    pub doc: Option<&'a mut Doc>,
    pub derives: Option<&'a mut Vec<Derive>>,
}

pub(super) fn parse_doc(attrs: &[Attribute]) -> Result<Doc> {
    let mut doc = Doc::new();
    parse(
        attrs,
        Parser {
            doc: Some(&mut doc),
            ..Parser::default()
        },
    )?;
    Ok(doc)
}

pub(super) fn parse(attrs: &[Attribute], mut parser: Parser) -> Result<()> {
    for attr in attrs {
        if attr.path.is_ident("doc") {
            if let Some(doc) = &mut parser.doc {
                let lit = parse_doc_attribute.parse2(attr.tokens.clone())?;
                doc.push(lit);
                continue;
            }
        } else if attr.path.is_ident("derive") {
            if let Some(derives) = &mut parser.derives {
                derives.extend(attr.parse_args_with(parse_derive_attribute)?);
                continue;
            }
        }
        return Err(Error::new_spanned(attr, "unsupported attribute"));
    }
    Ok(())
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

impl Derive {
    pub fn from(ident: &Ident) -> Option<Self> {
        match ident.to_string().as_str() {
            "Clone" => Some(Derive::Clone),
            "Copy" => Some(Derive::Copy),
            _ => None,
        }
    }
}

impl AsRef<str> for Derive {
    fn as_ref(&self) -> &str {
        match self {
            Derive::Clone => "Clone",
            Derive::Copy => "Copy",
        }
    }
}
