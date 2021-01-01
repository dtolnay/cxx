use crate::syntax::{Lifetimes, NamedType, Pair, Symbol};
use proc_macro2::{Ident, Span};
use std::iter;
use syn::punctuated::Punctuated;

impl Pair {
    pub fn to_symbol(&self) -> Symbol {
        Symbol::from_idents(self.iter_all_segments())
    }

    pub fn to_fully_qualified(&self) -> String {
        let mut fully_qualified = String::new();
        for segment in self.iter_all_segments() {
            fully_qualified += "::";
            fully_qualified += &segment.to_string();
        }
        fully_qualified
    }

    fn iter_all_segments(&self) -> impl Iterator<Item = &Ident> {
        self.namespace.iter().chain(iter::once(&self.cxx))
    }
}

impl NamedType {
    pub fn new(rust: Ident) -> Self {
        let generics = Lifetimes {
            lt_token: None,
            lifetimes: Punctuated::new(),
            gt_token: None,
        };
        NamedType { rust, generics }
    }

    pub fn span(&self) -> Span {
        self.rust.span()
    }
}
