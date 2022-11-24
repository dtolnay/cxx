use crate::syntax::{NamedType, Ty1, Ty2, Type};
use proc_macro2::{Ident, Span};
use std::hash::{Hash, Hasher};
use syn::Token;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum ImplKey<'a> {
    RustBox(NamedImplKey<'a>),
    RustVec(NamedImplKey<'a>),
    UniquePtr(NamedImplKey<'a>),
    SharedPtr(NamedImplKey<'a>),
    WeakPtr(NamedImplKey<'a>),
    CxxVector(NamedImplKey<'a>),
    CxxFunction(DoubleNamedImplKey<'a>),
}

#[derive(Copy, Clone)]
pub struct NamedImplKey<'a> {
    pub begin_span: Span,
    pub rust: &'a Ident,
    pub lt_token: Option<Token![<]>,
    pub gt_token: Option<Token![>]>,
    pub end_span: Span,
}

#[derive(Copy, Clone)]
pub struct DoubleNamedImplKey<'a> {
    pub begin_span: Span,
    pub id1_ampersand: Option<Token![&]>,
    pub id1: &'a Ident,
    pub id1_lt_token: Option<Token![<]>,
    pub id1_gt_token: Option<Token![>]>,
    pub id2: Option<&'a Ident>,
    pub id2_lt_token: Option<Token![<]>,
    pub id2_gt_token: Option<Token![>]>,
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
        } else if let Type::CxxFunction(ty) = self {
            let ret: Option<&NamedType> = if let Type::Ident(ret) = &ty.second {
                Some(ret)
            } else if let Type::Void(_) = &ty.second {
                None
            } else {
                return None;
            };

            if let Type::Ident(args) = &ty.first {
                return Some(ImplKey::CxxFunction(DoubleNamedImplKey::new(ty, None, args, ret)));
            } else if let Type::Ref(rf) = &ty.first {
                if let Type::Ident(args) = &rf.inner {
                    return Some(ImplKey::CxxFunction(DoubleNamedImplKey::new(ty, Some(rf.ampersand), args, ret)));
                }
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

impl<'a> PartialEq for DoubleNamedImplKey<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.id1_ampersand.is_some() == other.id1_ampersand.is_some()
            && PartialEq::eq(&self.id1, &other.id1)
            && PartialEq::eq(&self.id2, &other.id2)
    }
}

impl<'a> Eq for DoubleNamedImplKey<'a> {}

impl<'a> Hash for DoubleNamedImplKey<'a> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.id1_ampersand.is_some().hash(hasher);
        self.id1.hash(hasher);
        self.id2.hash(hasher);
    }
}

impl<'a> DoubleNamedImplKey<'a> {
    fn new(outer: &Ty2, ampersand: Option<Token![&]>, first: &'a NamedType, second: Option<&'a NamedType>) -> Self {
        DoubleNamedImplKey {
            begin_span: outer.name.span(),
            id1_ampersand: ampersand,
            id1: &first.rust,
            id1_lt_token: first.generics.lt_token,
            id1_gt_token: first.generics.gt_token,
            id2: second.map(|s| &s.rust),
            id2_lt_token: second.map(|s| s.generics.lt_token).flatten(),
            id2_gt_token: second.map(|s| s.generics.gt_token).flatten(),
            end_span: outer.rangle.span,
        }
    }
}
