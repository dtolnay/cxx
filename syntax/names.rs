use crate::syntax::{Pair, RustName, Symbol, Types};
use proc_macro2::{Ident, Span};
use std::iter;

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

impl RustName {
    pub fn new(rust: Ident) -> Self {
        RustName { rust }
    }

    pub fn from_ref(rust: &Ident) -> &Self {
        unsafe { &*(rust as *const Ident as *const Self) }
    }

    pub fn span(&self) -> Span {
        self.rust.span()
    }

    pub fn to_symbol(&self, types: &Types) -> Symbol {
        types.resolve(self).to_symbol()
    }
}
