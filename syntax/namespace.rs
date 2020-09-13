use crate::syntax::qualified::QualifiedName;
use quote::IdentFragment;
use std::fmt::{self, Display};
use std::slice::Iter;
use syn::parse::{Parse, ParseStream, Result};
use syn::{Ident, Token};

mod kw {
    syn::custom_keyword!(namespace);
}

#[derive(Clone)]
pub struct Namespace {
    segments: Vec<Ident>,
}

impl Namespace {
    pub fn none() -> Self {
        Namespace {
            segments: Vec::new(),
        }
    }

    pub fn iter(&self) -> Iter<Ident> {
        self.segments.iter()
    }

    pub fn path_for_type(&self, ident: &Ident) -> String {
        let mut segments = self.iter().map(ToString::to_string).collect::<Vec<_>>();
        segments.push(ident.to_string());
        segments.join("::")
    }
}

impl From<QualifiedName> for Namespace {
    fn from(value: QualifiedName) -> Namespace {
        Namespace {
            segments: value.segments,
        }
    }
}

impl Parse for Namespace {
    fn parse(input: ParseStream) -> Result<Self> {
        if !input.is_empty() {
            input.parse::<kw::namespace>()?;
            input.parse::<Token![=]>()?;
            let name = input.call(QualifiedName::parse_quoted_or_unquoted)?;
            input.parse::<Option<Token![,]>>()?;
            return Ok(Namespace::from(name));
        }
        Ok(Namespace::none())
    }
}

impl Display for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for segment in self {
            write!(f, "{}$", segment)?;
        }
        Ok(())
    }
}

impl IdentFragment for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl<'a> IntoIterator for &'a Namespace {
    type Item = &'a Ident;
    type IntoIter = Iter<'a, Ident>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
