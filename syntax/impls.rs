use crate::syntax::{ExternFn, Receiver, Ref, Signature, Ty1, Type};
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
            Type::Fn(t) => t.hash(state),
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
            (Type::Fn(lhs), Type::Fn(rhs)) => lhs == rhs,
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
            mutability,
            inner,
        } = self;
        let Ref {
            ampersand: _,
            mutability: mutability2,
            inner: inner2,
        } = other;
        mutability.is_some() == mutability2.is_some() && inner == inner2
    }
}

impl Hash for Ref {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let Ref {
            ampersand: _,
            mutability,
            inner,
        } = self;
        mutability.is_some().hash(state);
        inner.hash(state);
    }
}

impl Eq for Signature {}

impl PartialEq for Signature {
    fn eq(&self, other: &Signature) -> bool {
        let Signature {
            fn_token: _,
            receiver,
            args,
            ret,
            throws,
        } = self;
        let Signature {
            fn_token: _,
            receiver: receiver2,
            args: args2,
            ret: ret2,
            throws: throws2,
        } = other;
        receiver == receiver2 && args == args2 && ret == ret2 && throws == throws2
    }
}

impl Hash for Signature {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let Signature {
            fn_token: _,
            receiver,
            args,
            ret,
            throws,
        } = self;
        receiver.hash(state);
        args.hash(state);
        ret.hash(state);
        throws.hash(state);
    }
}

impl Eq for Receiver {}

impl PartialEq for Receiver {
    fn eq(&self, other: &Receiver) -> bool {
        let Receiver { mutability, ident } = self;
        let Receiver {
            mutability: mutability2,
            ident: ident2,
        } = other;
        mutability.is_some() == mutability2.is_some() && ident == ident2
    }
}

impl Hash for Receiver {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let Receiver { mutability, ident } = self;
        mutability.is_some().hash(state);
        ident.hash(state);
    }
}
