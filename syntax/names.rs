use crate::syntax::{Namespace, Pair, RustName, Symbol, Types};
use proc_macro2::{Ident, Span};
use std::iter;

impl Pair {
    pub fn new(namespace: Namespace, cxx: Ident, rust: Ident) -> Self {
        Self {
            namespace,
            cxx,
            rust,
        }
    }

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
    pub fn new(ident: Ident) -> Self {
        Self { rust: ident }
    }

    pub fn from_ref(ident: &Ident) -> &Self {
        unsafe { &*(ident as *const Ident as *const Self) }
    }

    pub fn span(&self) -> Span {
        self.rust.span()
    }

    pub fn to_symbol(&self, types: &Types) -> Symbol {
        types.resolve(self).to_symbol()
    }
}
