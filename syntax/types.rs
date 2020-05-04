use crate::syntax::atom::Atom::{self, *};
use crate::syntax::set::OrderedSet as Set;
use crate::syntax::{Api, Derive, Enum, ExternType, Struct, Type};
use proc_macro2::Ident;
use quote::quote;
use std::collections::{BTreeMap as Map, HashSet as UnorderedSet};
use syn::{Error, Result};

pub struct Types<'a> {
    pub all: Set<'a, Type>,
    pub structs: Map<Ident, &'a Struct>,
    pub enums: Map<Ident, &'a Enum>,
    pub cxx: Set<'a, Ident>,
    pub rust: Set<'a, Ident>,
}

impl<'a> Types<'a> {
    pub fn collect(apis: &'a [Api]) -> Result<Self> {
        let mut all = Set::new();
        let mut structs = Map::new();
        let mut enums = Map::new();
        let mut cxx = Set::new();
        let mut rust = Set::new();

        fn visit<'a>(all: &mut Set<'a, Type>, ty: &'a Type) {
            all.insert(ty);
            match ty {
                Type::Ident(_) | Type::Str(_) | Type::Void(_) | Type::SliceRefU8(_) => {}
                Type::RustBox(ty)
                | Type::UniquePtr(ty)
                | Type::CxxVector(ty)
                | Type::RustVec(ty) => visit(all, &ty.inner),
                Type::Ref(r) => visit(all, &r.inner),
                Type::Slice(s) => visit(all, &s.inner),
                Type::Fn(f) => {
                    if let Some(ret) = &f.ret {
                        visit(all, ret);
                    }
                    for arg in &f.args {
                        visit(all, &arg.ty);
                    }
                }
            }
        }

        let mut type_names = UnorderedSet::new();
        for api in apis {
            match api {
                Api::Include(_) => {}
                Api::Struct(strct) => {
                    if !type_names.insert(&strct.ident) {
                        return Err(duplicate_struct(strct));
                    }
                    structs.insert(strct.ident.clone(), strct);
                    for field in &strct.fields {
                        visit(&mut all, &field.ty);
                    }
                }
                Api::Enum(enm) => {
                    if !type_names.insert(&enm.ident) {
                        return Err(duplicate_enum(enm));
                    }
                    enums.insert(enm.ident.clone(), enm);
                }
                Api::CxxType(ety) => {
                    if !type_names.insert(&ety.ident) {
                        return Err(duplicate_type(ety));
                    }
                    cxx.insert(&ety.ident);
                }
                Api::RustType(ety) => {
                    if !type_names.insert(&ety.ident) {
                        return Err(duplicate_type(ety));
                    }
                    rust.insert(&ety.ident);
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
            enums,
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
            Type::RustVec(_) => true,
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

fn duplicate_enum(enm: &Enum) -> Error {
    let enum_token = enm.enum_token;
    let ident = &enm.ident;
    let range = quote!(#enum_token #ident);
    Error::new_spanned(range, "duplicate type")
}

fn duplicate_type(ety: &ExternType) -> Error {
    let type_token = ety.type_token;
    let ident = &ety.ident;
    let range = quote!(#type_token #ident);
    Error::new_spanned(range, "duplicate type")
}
