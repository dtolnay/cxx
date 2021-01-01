use crate::syntax::Type;
use proc_macro2::Ident;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum ImplKey<'a> {
    RustBox(&'a Ident),
    RustVec(&'a Ident),
    UniquePtr(&'a Ident),
    SharedPtr(&'a Ident),
    WeakPtr(&'a Ident),
    CxxVector(&'a Ident),
}

impl Type {
    pub(crate) fn impl_key(&self) -> Option<ImplKey> {
        if let Type::RustBox(ty) = self {
            if let Type::Ident(ident) = &ty.inner {
                return Some(ImplKey::RustBox(&ident.rust));
            }
        } else if let Type::RustVec(ty) = self {
            if let Type::Ident(ident) = &ty.inner {
                return Some(ImplKey::RustVec(&ident.rust));
            }
        } else if let Type::UniquePtr(ty) = self {
            if let Type::Ident(ident) = &ty.inner {
                return Some(ImplKey::UniquePtr(&ident.rust));
            }
        } else if let Type::SharedPtr(ty) = self {
            if let Type::Ident(ident) = &ty.inner {
                return Some(ImplKey::SharedPtr(&ident.rust));
            }
        } else if let Type::WeakPtr(ty) = self {
            if let Type::Ident(ident) = &ty.inner {
                return Some(ImplKey::WeakPtr(&ident.rust));
            }
        } else if let Type::CxxVector(ty) = self {
            if let Type::Ident(ident) = &ty.inner {
                return Some(ImplKey::CxxVector(&ident.rust));
            }
        }
        None
    }
}
