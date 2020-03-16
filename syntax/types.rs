use crate::syntax::atom::Atom::{self, *};
use crate::syntax::set::OrderedSet as Set;
use crate::syntax::{Api, Derive, ExternType, Struct, Type};
use proc_macro2::Ident;
use quote::quote;
use std::collections::BTreeMap as Map;
use syn::{Error, Result};

pub struct Types<'a> {
    pub all: Set<'a, Type>,
    pub structs: Map<Ident, &'a Struct>,
    pub cxx: Set<'a, Ident>,
    pub rust: Set<'a, Ident>,
}

impl<'a> Types<'a> {
    pub fn collect(apis: &'a [Api]) -> Result<Self> {
        let mut all = Set::new();
        let mut structs = Map::new();
        let mut cxx = Set::new();
        let mut rust = Set::new();

        fn visit<'a>(all: &mut Set<'a, Type>, ty: &'a Type) {
            all.insert(ty);
            match ty {
                Type::Ident(_) | Type::Str(_) | Type::Void(_) => {}
                Type::RustBox(ty) | Type::UniquePtr(ty) => visit(all, &ty.inner),
                Type::Ref(r) => visit(all, &r.inner),
            }
        }

        for api in apis {
            match api {
                Api::Include(_) => {}
                Api::Struct(strct) => {
                    let ident = &strct.ident;
                    if structs.contains_key(ident) || cxx.contains(ident) || rust.contains(ident) {
                        return Err(duplicate_struct(strct));
                    }
                    structs.insert(strct.ident.clone(), strct);
                    for field in &strct.fields {
                        visit(&mut all, &field.ty);
                    }
                }
                Api::CxxType(ety) => {
                    let ident = &ety.ident;
                    if structs.contains_key(ident) || cxx.contains(ident) || rust.contains(ident) {
                        return Err(duplicate_type(ety));
                    }
                    cxx.insert(ident);
                }
                Api::RustType(ety) => {
                    let ident = &ety.ident;
                    if structs.contains_key(ident) || cxx.contains(ident) || rust.contains(ident) {
                        return Err(duplicate_type(ety));
                    }
                    rust.insert(ident);
                }
                Api::CxxFunction(efn) | Api::RustFunction(efn) => {
                    for arg in &efn.args {
                        visit(&mut all, &arg.ty);
                    }
                    if let Some(ret) = &efn.ret {
                        visit(&mut all, ret);
                    }
                }
            }
        }

        Ok(Types {
            all,
            structs,
            cxx,
            rust,
        })
    }

    pub fn needs_indirect_abi(&self, ty: &Type) -> bool {
        match ty {
            Type::Ident(ident) => {
                if let Some(strct) = self.structs.get(ident) {
                    !self.is_pod(strct)
                } else {
                    Atom::from(ident) == Some(RustString)
                }
            }
            _ => false,
        }
    }

    pub fn is_pod(&self, strct: &Struct) -> bool {
        for derive in &strct.derives {
            if *derive == Derive::Copy {
                return true;
            }
        }
        false
    }
}

impl<'t, 'a> IntoIterator for &'t Types<'a> {
    type Item = &'a Type;
    type IntoIter = crate::syntax::set::Iter<'t, 'a, Type>;
    fn into_iter(self) -> Self::IntoIter {
        self.all.into_iter()
    }
}

fn duplicate_struct(strct: &Struct) -> Error {
    let struct_token = strct.struct_token;
    let ident = &strct.ident;
    let range = quote!(#struct_token #ident);
    Error::new_spanned(range, "duplicate type")
}

fn duplicate_type(ety: &ExternType) -> Error {
    let type_token = ety.type_token;
    let ident = &ety.ident;
    let range = quote!(#type_token #ident);
    Error::new_spanned(range, "duplicate type")
}
