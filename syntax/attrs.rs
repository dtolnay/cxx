use crate::syntax::{Derive, Doc};
use proc_macro2::Ident;
use syn::parse::{ParseStream, Parser};
use syn::{Attribute, Error, LitStr, Path, Result, Token};

pub(super) fn parse_doc(attrs: &[Attribute]) -> Result<Doc> {
    let mut doc = Doc::new();
    let derives = None;
    parse(attrs, &mut doc, derives)?;
    Ok(doc)
}

pub(super) fn parse(
    attrs: &[Attribute],
    doc: &mut Doc,
    mut derives: Option<&mut Vec<Derive>>,
) -> Result<()> {
    for attr in attrs {
        if attr.path.is_ident("doc") {
            let lit = parse_doc_attribute.parse2(attr.tokens.clone())?;
            doc.push(lit);
            continue;
        } else if attr.path.is_ident("derive") {
            if let Some(derives) = &mut derives {
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
