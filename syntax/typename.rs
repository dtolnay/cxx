use crate::syntax::namespace::Namespace;
use crate::syntax::{Atom, Type};

pub fn to_typename(namespace: &Namespace, ty: &Type) -> String {
    match ty {
        Type::Ident(ident) => {
            let mut inner = String::new();
            // Do not apply namespace to built-in type
            let is_user_type = Atom::from(ident).is_none();
            if is_user_type {
                for name in namespace {
                    inner += name;
                    inner += "::";
                }
            }
            if let Some(ti) = Atom::from(ident) {
                inner += ti.to_cxx();
            } else {
                inner += &ident.to_string();
            };
            inner
        }
        Type::RustBox(ptr) => format!("rust_box<{}>", to_typename(namespace, &ptr.inner)),
        Type::RustVec(ptr) => format!("rust_vec<{}>", to_typename(namespace, &ptr.inner)),
        Type::UniquePtr(ptr) => {
            format!("::std::unique_ptr<{}>", to_typename(namespace, &ptr.inner))
        }
        Type::CxxVector(ptr) => format!("::std::vector<{}>", to_typename(namespace, &ptr.inner)),
        _ => unimplemented!(),
    }
}
