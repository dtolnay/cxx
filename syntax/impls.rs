use crate::syntax::{ExternFn, Ref, Signature, Ty1, Type};
use std::hash::{Hash, Hasher};
use std::mem;
use std::ops::Deref;

impl Deref for ExternFn {
    type Target = Signature;

    fn deref(&self) -> &Self::Target {
        &self.sig
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
            (Type::Void(_), Type::Void(_)) => true,
            (_, _) => false,
        }
    }
}

impl Eq for Ty1 {}

impl PartialEq for Ty1 {
    fn eq(&self, other: &Ty1) -> bool {
        self.name == other.name && self.inner == other.inner
    }
}

impl Hash for Ty1 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.inner.hash(state);
    }
}

impl Eq for Ref {}

impl PartialEq for Ref {
    fn eq(&self, other: &Ref) -> bool {
        self.inner == other.inner
    }
}

impl Hash for Ref {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}
