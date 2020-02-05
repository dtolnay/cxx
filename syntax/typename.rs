use crate::syntax::{Atom, Type};

pub trait ToTypename {
    fn to_typename(&self, namespace: &Vec<String>) -> String;
}

impl ToTypename for Type {
    fn to_typename(&self, namespace: &Vec<String>) -> String {
        match self {
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
            Type::RustBox(ptr) => format!("rust_box<{}>", ptr.inner.to_typename(namespace)),
            Type::RustVec(ptr) => format!("rust_vec<{}>", ptr.inner.to_typename(namespace)),
            Type::UniquePtr(ptr) => {
                format!("std::unique_ptr<{}>", ptr.inner.to_typename(namespace))
            }
            Type::Vector(ptr) => format!("std::vector<{}>", ptr.inner.to_typename(namespace)),
            _ => unimplemented!(),
        }
    }
}
