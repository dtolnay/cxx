use crate::syntax::instantiate::{DoubleNamedImplKey, NamedImplKey};
use crate::syntax::{Lifetime, Lifetimes, NamedType, Pair, Type, Types, TupleStruct};
use proc_macro2::Ident;

#[derive(Copy, Clone)]
pub struct Resolution<'a> {
    pub name: &'a Pair,
    pub generics: &'a Lifetimes,
}

impl<'a> Types<'a> {
    pub fn resolve_tuple_struct(&self, ident: &impl UnresolvedName) -> Option<&TupleStruct> {
        self.tuple_structs.get(ident.ident()).map(|t| *t)
    }

    pub fn resolve_cxx_arg_type(&self, key: &DoubleNamedImplKey) -> (Option<&Type>, Option<&Lifetime>) {
        for t in self.all.iter() {
            if let Type::Ident(t_ident) = t {
                if &t_ident.rust == key.id1 && key.id1_ampersand.is_none() {
                    return (Some(t), None)
                }
            } else if let Type::Ref(t_ref) = t {
                if let Type::Ident(t_ident) = &t_ref.inner {
                    if &t_ident.rust == key.id1 && key.id1_ampersand.is_some() {
                        return (Some(t), t_ref.lifetime.as_ref())
                    }
                }
            }
        }
        (None, None)
    }

    pub fn resolve(&self, ident: &impl UnresolvedName) -> Resolution<'a> {
        let ident = ident.ident();
        match self.try_resolve(ident) {
            Some(resolution) => resolution,
            None => panic!("Unable to resolve type `{}`", ident),
        }
    }

    pub fn try_resolve(&self, ident: &impl UnresolvedName) -> Option<Resolution<'a>> {
        let ident = ident.ident();
        self.resolutions.get(ident).copied()
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

impl<'a> UnresolvedName for NamedImplKey<'a> {
    fn ident(&self) -> &Ident {
        self.rust
    }
}
