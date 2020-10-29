use crate::syntax::{
    Api, CppName, ExternFn, Impl, Include, Namespace, Pair, Receiver, Ref, ResolvableName,
    Signature, Slice, Symbol, Ty1, Type, Types,
};
use proc_macro2::{Ident, Span};
use std::borrow::Borrow;
use std::hash::{Hash, Hasher};
use std::mem;
use std::ops::{Deref, DerefMut};
use syn::Token;

impl PartialEq for Include {
    fn eq(&self, other: &Include) -> bool {
        let Include {
            path,
            kind,
            begin_span: _,
            end_span: _,
        } = self;
        let Include {
            path: path2,
            kind: kind2,
            begin_span: _,
            end_span: _,
        } = other;
        path == path2 && kind == kind2
    }
}

impl Deref for ExternFn {
    type Target = Signature;

    fn deref(&self) -> &Self::Target {
        &self.sig
    }
}

impl DerefMut for ExternFn {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sig
    }
}

impl Hash for Type {
    fn hash<H: Hasher>(&self, state: &mut H) {
        mem::discriminant(self).hash(state);
        match self {
            Type::Ident(t) => t.hash(state),
            Type::RustBox(t) => t.hash(state),
            Type::UniquePtr(t) => t.hash(state),
            Type::Ref(t) => t.hash(state),
            Type::Str(t) => t.hash(state),
            Type::RustVec(t) => t.hash(state),
            Type::CxxVector(t) => t.hash(state),
            Type::Fn(t) => t.hash(state),
            Type::Slice(t) => t.hash(state),
            Type::SliceRefU8(t) => t.hash(state),
            Type::Void(_) => {}
        }
    }
}

impl Eq for Type {}

impl PartialEq for Type {
    fn eq(&self, other: &Type) -> bool {
        match (self, other) {
            (Type::Ident(lhs), Type::Ident(rhs)) => lhs == rhs,
            (Type::RustBox(lhs), Type::RustBox(rhs)) => lhs == rhs,
            (Type::UniquePtr(lhs), Type::UniquePtr(rhs)) => lhs == rhs,
            (Type::Ref(lhs), Type::Ref(rhs)) => lhs == rhs,
            (Type::Str(lhs), Type::Str(rhs)) => lhs == rhs,
            (Type::RustVec(lhs), Type::RustVec(rhs)) => lhs == rhs,
            (Type::CxxVector(lhs), Type::CxxVector(rhs)) => lhs == rhs,
            (Type::Fn(lhs), Type::Fn(rhs)) => lhs == rhs,
            (Type::Slice(lhs), Type::Slice(rhs)) => lhs == rhs,
            (Type::SliceRefU8(lhs), Type::SliceRefU8(rhs)) => lhs == rhs,
            (Type::Void(_), Type::Void(_)) => true,
            (_, _) => false,
        }
    }
}

impl Eq for Ty1 {}

impl PartialEq for Ty1 {
    fn eq(&self, other: &Ty1) -> bool {
        let Ty1 {
            name,
            langle: _,
            inner,
            rangle: _,
        } = self;
        let Ty1 {
            name: name2,
            langle: _,
            inner: inner2,
            rangle: _,
        } = other;
        name == name2 && inner == inner2
    }
}

impl Hash for Ty1 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let Ty1 {
            name,
            langle: _,
            inner,
            rangle: _,
        } = self;
        name.hash(state);
        inner.hash(state);
    }
}

impl Eq for Ref {}

impl PartialEq for Ref {
    fn eq(&self, other: &Ref) -> bool {
        let Ref {
            ampersand: _,
            lifetime,
            mutability,
            inner,
        } = self;
        let Ref {
            ampersand: _,
            lifetime: lifetime2,
            mutability: mutability2,
            inner: inner2,
        } = other;
        lifetime == lifetime2 && mutability.is_some() == mutability2.is_some() && inner == inner2
    }
}

impl Hash for Ref {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let Ref {
            ampersand: _,
            lifetime,
            mutability,
            inner,
        } = self;
        lifetime.hash(state);
        mutability.is_some().hash(state);
        inner.hash(state);
    }
}

impl Eq for Slice {}

impl PartialEq for Slice {
    fn eq(&self, other: &Slice) -> bool {
        let Slice { bracket: _, inner } = self;
        let Slice {
            bracket: _,
            inner: inner2,
        } = other;
        inner == inner2
    }
}

impl Hash for Slice {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let Slice { bracket: _, inner } = self;
        inner.hash(state);
    }
}

impl Eq for Signature {}

impl PartialEq for Signature {
    fn eq(&self, other: &Signature) -> bool {
        let Signature {
            unsafety,
            fn_token: _,
            receiver,
            args,
            ret,
            throws,
            paren_token: _,
            throws_tokens: _,
        } = self;
        let Signature {
            unsafety: unsafety2,
            fn_token: _,
            receiver: receiver2,
            args: args2,
            ret: ret2,
            throws: throws2,
            paren_token: _,
            throws_tokens: _,
        } = other;
        unsafety.is_some() == unsafety2.is_some()
            && receiver == receiver2
            && ret == ret2
            && throws == throws2
            && args.len() == args2.len()
            && args.iter().zip(args2).all(|(arg, arg2)| arg == arg2)
    }
}

impl Hash for Signature {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let Signature {
            unsafety,
            fn_token: _,
            receiver,
            args,
            ret,
            throws,
            paren_token: _,
            throws_tokens: _,
        } = self;
        unsafety.is_some().hash(state);
        receiver.hash(state);
        for arg in args {
            arg.hash(state);
        }
        ret.hash(state);
        throws.hash(state);
    }
}

impl Eq for Receiver {}

impl PartialEq for Receiver {
    fn eq(&self, other: &Receiver) -> bool {
        let Receiver {
            ampersand: _,
            lifetime,
            mutability,
            var: _,
            ty,
            shorthand: _,
        } = self;
        let Receiver {
            ampersand: _,
            lifetime: lifetime2,
            mutability: mutability2,
            var: _,
            ty: ty2,
            shorthand: _,
        } = other;
        lifetime == lifetime2 && mutability.is_some() == mutability2.is_some() && ty == ty2
    }
}

impl Hash for Receiver {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let Receiver {
            ampersand: _,
            lifetime,
            mutability,
            var: _,
            ty,
            shorthand: _,
        } = self;
        lifetime.hash(state);
        mutability.is_some().hash(state);
        ty.hash(state);
    }
}

impl Hash for Impl {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let Impl {
            impl_token: _,
            ty,
            brace_token: _,
        } = self;
        ty.hash(state);
    }
}

impl Eq for Impl {}

impl PartialEq for Impl {
    fn eq(&self, other: &Impl) -> bool {
        let Impl {
            impl_token: _,
            ty,
            brace_token: _,
        } = self;
        let Impl {
            impl_token: _,
            ty: ty2,
            brace_token: _,
        } = other;
        ty == ty2
    }
}

impl Borrow<Type> for &Impl {
    fn borrow(&self) -> &Type {
        &self.ty
    }
}

impl Pair {
    /// Use this constructor when the item can't have a different
    /// name in Rust and C++. For cases where #[rust_name] and similar
    /// attributes can be used, construct the object by hand.
    pub fn new(ns: Namespace, ident: Ident) -> Self {
        Self {
            rust: ident.clone(),
            cxx: CppName::new(ns, ident),
        }
    }
}

impl ResolvableName {
    pub fn new(ident: Ident) -> Self {
        Self { rust: ident }
    }

    pub fn from_pair(pair: Pair) -> Self {
        Self { rust: pair.rust }
    }

    pub fn make_self(span: Span) -> Self {
        Self {
            rust: Token![Self](span).into(),
        }
    }

    pub fn is_self(&self) -> bool {
        self.rust == "Self"
    }

    pub fn span(&self) -> Span {
        self.rust.span()
    }

    pub fn to_symbol(&self, types: &Types) -> Symbol {
        types.resolve(self).to_symbol()
    }
}

impl Api {
    pub fn get_namespace(&self) -> Option<&Namespace> {
        match self {
            Api::CxxFunction(cfn) => Some(&cfn.ident.cxx.ns),
            Api::CxxType(cty) => Some(&cty.ident.cxx.ns),
            Api::Enum(enm) => Some(&enm.ident.cxx.ns),
            Api::Struct(strct) => Some(&strct.ident.cxx.ns),
            Api::RustType(rty) => Some(&rty.ident.cxx.ns),
            Api::RustFunction(rfn) => Some(&rfn.ident.cxx.ns),
            Api::Impl(_) | Api::Include(_) | Api::TypeAlias(_) => None,
        }
    }
}

impl CppName {
    pub fn new(ns: Namespace, ident: Ident) -> Self {
        Self { ns, ident }
    }

    fn iter_all_segments(
        &self,
    ) -> std::iter::Chain<std::slice::Iter<Ident>, std::iter::Once<&Ident>> {
        self.ns.iter().chain(std::iter::once(&self.ident))
    }

    fn join(&self, sep: &str) -> String {
        self.iter_all_segments()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join(sep)
    }

    pub fn to_symbol(&self) -> Symbol {
        Symbol::from_idents(self.iter_all_segments())
    }

    pub fn to_fully_qualified(&self) -> String {
        format!("::{}", self.join("::"))
    }
}
