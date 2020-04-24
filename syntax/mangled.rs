use crate::syntax::{Atom, Type};

pub trait ToMangled {
    fn to_mangled(&self, namespace: &Vec<String>) -> String;
}

impl ToMangled for Type {
    fn to_mangled(&self, namespace: &Vec<String>) -> String {
        match self {
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
            Type::RustBox(ptr) => format!("rust_box${}", ptr.inner.to_mangled(namespace)),
            Type::RustVec(ptr) => format!("rust_vec${}", ptr.inner.to_mangled(namespace)),
            Type::UniquePtr(ptr) => format!("std$unique_ptr${}", ptr.inner.to_mangled(namespace)),
            Type::CxxVector(ptr) => format!("std$vector${}", ptr.inner.to_mangled(namespace)),
            _ => unimplemented!(),
        }
    }
}
