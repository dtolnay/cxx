use crate::syntax::{NamedType, Ty1, Type};
use proc_macro2::{Ident, Span};
use std::hash::{Hash, Hasher};
use syn::Token;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) enum ImplKey<'a> {
    RustBox(NamedImplKey<'a>),
    RustVec(NamedImplKey<'a>),
    RustOption(OptionInner<'a>),
    UniquePtr(NamedImplKey<'a>),
    SharedPtr(NamedImplKey<'a>),
    WeakPtr(NamedImplKey<'a>),
    CxxVector(NamedImplKey<'a>),
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) enum OptionInner<'a> {
    RustBox(NamedImplKey<'a>),
    Ref(NamedImplKey<'a>),
    MutRef(NamedImplKey<'a>),
}

#[derive(Copy, Clone)]
pub(crate) struct NamedImplKey<'a> {
    #[allow(dead_code)] // only used by cxxbridge-macro, not cxx-build
    pub begin_span: Span,
    pub rust: &'a Ident,
    #[allow(dead_code)] // only used by cxxbridge-macro, not cxx-build
    pub lt_token: Option<Token![<]>,
    #[allow(dead_code)] // only used by cxxbridge-macro, not cxx-build
    pub gt_token: Option<Token![>]>,
    #[allow(dead_code)] // only used by cxxbridge-macro, not cxx-build
    pub end_span: Span,
}

impl Type {
    pub(crate) fn impl_key(&self) -> Option<ImplKey> {
        if let Type::RustBox(ty) = self {
            if let Type::Ident(ident) = &ty.inner {
                return Some(ImplKey::RustBox(NamedImplKey::new(ty, ident)));
            }
        } else if let Type::RustVec(ty) = self {
            if let Type::Ident(ident) = &ty.inner {
                return Some(ImplKey::RustVec(NamedImplKey::new(ty, ident)));
            }
        } else if let Type::RustOption(ty) = self {
            match &ty.inner {
                Type::RustBox(_) => {
                    let impl_key = ty.inner.impl_key()?;
                    match impl_key {
                        ImplKey::RustBox(named) => {
                            return Some(ImplKey::RustOption(OptionInner::RustBox(named)))
                        }
                        _ => unreachable!(),
                    }
                }
                Type::Ref(r) => match &r.inner {
                    Type::Ident(ident) => {
                        if r.mutable {
                            return Some(ImplKey::RustOption(OptionInner::MutRef(
                                NamedImplKey::new(ty, ident),
                            )));
                        } else {
                            return Some(ImplKey::RustOption(OptionInner::Ref(NamedImplKey::new(
                                ty, ident,
                            ))));
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        } else if let Type::UniquePtr(ty) = self {
            if let Type::Ident(ident) = &ty.inner {
                return Some(ImplKey::UniquePtr(NamedImplKey::new(ty, ident)));
            }
        } else if let Type::SharedPtr(ty) = self {
            if let Type::Ident(ident) = &ty.inner {
                return Some(ImplKey::SharedPtr(NamedImplKey::new(ty, ident)));
            }
        } else if let Type::WeakPtr(ty) = self {
            if let Type::Ident(ident) = &ty.inner {
                return Some(ImplKey::WeakPtr(NamedImplKey::new(ty, ident)));
            }
        } else if let Type::CxxVector(ty) = self {
            if let Type::Ident(ident) = &ty.inner {
                return Some(ImplKey::CxxVector(NamedImplKey::new(ty, ident)));
            }
        }
        None
    }
}

impl<'a> PartialEq for NamedImplKey<'a> {
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(self.rust, other.rust)
    }
}

impl<'a> Eq for NamedImplKey<'a> {}

impl<'a> Hash for NamedImplKey<'a> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.rust.hash(hasher);
    }
}

impl<'a> NamedImplKey<'a> {
    fn new(outer: &Ty1, inner: &'a NamedType) -> Self {
        NamedImplKey {
            begin_span: outer.name.span(),
            rust: &inner.rust,
            lt_token: inner.generics.lt_token,
            gt_token: inner.generics.gt_token,
            end_span: outer.rangle.span,
        }
    }
}
