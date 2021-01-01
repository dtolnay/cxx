use crate::syntax::{NamedType, Pair, Types};
use proc_macro2::Ident;

impl<'a> Types<'a> {
    pub fn resolve(&self, ident: &impl UnresolvedName) -> &Pair {
        self.resolutions
            .get(ident.ident())
            .expect("Unable to resolve type")
    }
}

pub trait UnresolvedName {
    fn ident(&self) -> &Ident;
}

impl UnresolvedName for Ident {
    fn ident(&self) -> &Ident {
        self
    }
}

impl UnresolvedName for NamedType {
    fn ident(&self) -> &Ident {
        &self.rust
    }
}
