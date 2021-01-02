use crate::syntax::{Lifetimes, NamedType, Pair, Types};
use proc_macro2::Ident;

#[derive(Copy, Clone)]
pub struct Resolution<'a> {
    pub name: &'a Pair,
    pub generics: &'a Lifetimes,
}

impl<'a> Types<'a> {
    pub fn resolve(&self, ident: &impl UnresolvedName) -> Resolution<'a> {
        *self
            .resolutions
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
