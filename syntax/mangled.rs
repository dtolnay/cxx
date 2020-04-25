use crate::syntax::namespace::Namespace;
use crate::syntax::{Atom, Type};

pub fn to_mangled(namespace: &Namespace, ty: &Type) -> String {
    match ty {
        Type::Ident(ident) => {
            let mut instance = String::new();
            // Do not apply namespace to built-in type
            let is_user_type = Atom::from(ident).is_none();
            if is_user_type {
                for name in namespace {
                    instance += name;
                    instance += "$";
                }
            }
            instance += &ident.to_string();
            instance
        }
        Type::RustBox(ptr) => format!("rust_box${}", to_mangled(namespace, &ptr.inner)),
        Type::RustVec(ptr) => format!("rust_vec${}", to_mangled(namespace, &ptr.inner)),
        Type::UniquePtr(ptr) => format!("std$unique_ptr${}", to_mangled(namespace, &ptr.inner)),
        Type::CxxVector(ptr) => format!("std$vector${}", to_mangled(namespace, &ptr.inner)),
        _ => unimplemented!(),
    }
}
