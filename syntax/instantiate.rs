use crate::syntax::types::Types;
use crate::syntax::{mangle, Symbol, Ty1, Type};
use proc_macro2::Span;
use std::hash::{Hash, Hasher};

#[derive(PartialEq, Eq, Hash)]
pub(crate) enum ImplKey<'a> {
    RustBox(NamedImplKey<'a>),
    RustVec(NamedImplKey<'a>),
    UniquePtr(NamedImplKey<'a>),
    SharedPtr(NamedImplKey<'a>),
    WeakPtr(NamedImplKey<'a>),
    CxxVector(NamedImplKey<'a>),
}

impl<'a> ImplKey<'a> {
    /// Whether to generate an implicit instantiation/monomorphization of a given generic type
    /// binding.  ("implicit" = without an explicit `impl Foo<T> {}` - see
    /// <https://cxx.rs/extern-c++.html?highlight=explicit#explicit-shim-trait-impls>).
    ///
    /// The main consideration is avoiding introducing conflicting/overlapping impls:
    ///
    /// * The `cxx` crate already provides impls for cases where `T` is a primitive
    ///   type like `u32`
    /// * Some generics (e.g. Rust bindings for C++ templates like `CxxVector<T>`, `UniquePtr<T>`,
    ///   etc.) require an `impl` of a `trait` provided by the `cxx` crate (such as
    ///   [`cxx::vector::VectorElement`] or [`cxx::memory::UniquePtrTarget`]).  To avoid violating
    ///   [Rust orphan rule](https://doc.rust-lang.org/reference/items/implementations.html#r-items.impl.trait.orphan-rule.intro)
    ///   we restrict `T` to be a local type
    ///   (TODO: or a fundamental type like `Box<LocalType>`).
    /// * Other generics (e.g. C++ bindings for Rust generics like `Vec<T>` or `Box<T>`)
    ///   don't necessarily need to follow the orphan rule, but we conservatively also
    ///   only generate implicit impls if `T` is a local type.  TODO: revisit?
    pub(crate) fn is_implicit_impl_ok(&self, types: &Types) -> bool {
        // TODO: relax this for Rust generics to allow Vec<Vec<T>> etc.
        types.is_local(self.inner())
    }

    /// Returns the generic type parameter `T` associated with `self`.
    /// For example, if `self` represents `UniquePtr<u32>` then this will return `u32`.
    pub(crate) fn inner(&self) -> &'a Type {
        let named_impl_key = match self {
            ImplKey::RustBox(key)
            | ImplKey::RustVec(key)
            | ImplKey::UniquePtr(key)
            | ImplKey::SharedPtr(key)
            | ImplKey::WeakPtr(key)
            | ImplKey::CxxVector(key) => key,
        };
        named_impl_key.inner
    }
}

pub(crate) struct NamedImplKey<'a> {
    #[cfg_attr(not(proc_macro), expect(dead_code))]
    pub begin_span: Span,
    /// Mangled form of the `outer` type.
    pub symbol: Symbol,
    /// Generic type - e.g. `UniquePtr<u8>`.
    #[allow(dead_code)] // only used by cxx-build, not cxxbridge-macro
    pub outer: &'a Type,
    /// Generic type argument - e.g. `u8` from `UniquePtr<u8>`.
    pub inner: &'a Type,
    #[allow(dead_code)] // only used by cxxbridge-macro, not cxx-build
    pub end_span: Span,
}

impl Type {
    pub(crate) fn impl_key(&self) -> Option<ImplKey> {
        match self {
            Type::RustBox(ty) => Some(ImplKey::RustBox(NamedImplKey::new(self, ty)?)),
            Type::RustVec(ty) => Some(ImplKey::RustVec(NamedImplKey::new(self, ty)?)),
            Type::UniquePtr(ty) => Some(ImplKey::UniquePtr(NamedImplKey::new(self, ty)?)),
            Type::SharedPtr(ty) => Some(ImplKey::SharedPtr(NamedImplKey::new(self, ty)?)),
            Type::WeakPtr(ty) => Some(ImplKey::WeakPtr(NamedImplKey::new(self, ty)?)),
            Type::CxxVector(ty) => Some(ImplKey::CxxVector(NamedImplKey::new(self, ty)?)),
            _ => None,
        }
    }
}

impl<'a> PartialEq for NamedImplKey<'a> {
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(&self.symbol, &other.symbol)
    }
}

impl<'a> Eq for NamedImplKey<'a> {}

impl<'a> Hash for NamedImplKey<'a> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.symbol.hash(hasher);
    }
}

impl<'a> NamedImplKey<'a> {
    fn new(outer: &'a Type, ty1: &'a Ty1) -> Option<Self> {
        let inner = &ty1.inner;
        Some(NamedImplKey {
            symbol: mangle::type_(inner)?,
            begin_span: ty1.name.span(),
            outer,
            inner,
            end_span: ty1.rangle.span,
        })
    }
}
