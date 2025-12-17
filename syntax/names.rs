use crate::syntax::symbol::Segment;
use crate::syntax::{Lifetimes, NamedType, Pair, Symbol};
use proc_macro2::{Ident, Span};
use std::fmt::{self, Display};
use std::iter;
use syn::ext::IdentExt;
use syn::parse::{Error, Parser, Result};
use syn::punctuated::Punctuated;

#[derive(Clone)]
pub(crate) struct ForeignName {
    text: String,
    span: Span,
}

impl Pair {
    pub(crate) fn to_symbol(&self) -> Symbol {
        let segments = self
            .namespace
            .iter()
            .map(|ident| ident as &dyn Segment)
            .chain(iter::once(&self.cxx as &dyn Segment));
        Symbol::from_idents(segments)
    }
}

impl NamedType {
    pub(crate) fn new(rust: Ident) -> Self {
        let generics = Lifetimes {
            lt_token: None,
            lifetimes: Punctuated::new(),
            gt_token: None,
        };
        NamedType { rust, generics }
    }
}

impl ForeignName {
    pub(crate) fn parse(text: &str, span: Span) -> Result<Self> {
        // TODO: support C++ names containing whitespace (`unsigned int`) or
        // non-alphanumeric characters (`operator++`).
        match Ident::parse_any.parse_str(text) {
            Ok(ident) => {
                let text = ident.to_string();
                Ok(ForeignName { text, span })
            }
            Err(err) => Err(Error::new(span, err)),
        }
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.text
    }

    pub(crate) fn span(&self) -> Span {
        self.span
    }
}

impl Display for ForeignName {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(&self.text)
    }
}

impl PartialEq<str> for ForeignName {
    fn eq(&self, rhs: &str) -> bool {
        self.text == rhs
    }
}
