use crate::syntax::atom::Atom::{self, *};
use crate::syntax::report::Errors;
use crate::syntax::set::OrderedSet as Set;
use crate::syntax::{Api, Derive, Enum, ExternType, Struct, Type, TypeAlias};
use proc_macro2::Ident;
use quote::ToTokens;
use std::collections::{BTreeMap as Map, HashSet as UnorderedSet};

pub struct Types<'a> {
    pub all: Set<&'a Type>,
    pub structs: Map<&'a Ident, &'a Struct>,
    pub enums: Map<&'a Ident, &'a Enum>,
    pub cxx: Set<&'a Ident>,
    pub rust: Set<&'a Ident>,
    pub aliases: Map<&'a Ident, &'a TypeAlias>,
    pub untrusted: Map<&'a Ident, &'a ExternType>,
}

impl<'a> Types<'a> {
    pub fn collect(cx: &mut Errors, apis: &'a [Api]) -> Self {
        let mut all = Set::new();
        let mut structs = Map::new();
        let mut enums = Map::new();
        let mut cxx = Set::new();
        let mut rust = Set::new();
        let mut aliases = Map::new();
        let mut untrusted = Map::new();

        fn visit<'a>(all: &mut Set<&'a Type>, ty: &'a Type) {
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
        let mut function_names = UnorderedSet::new();
        for api in apis {
            // The same identifier is permitted to be declared as both a shared
            // enum and extern C++ type, or shared struct and extern C++ type.
            // That indicates to not emit the C++ enum/struct definition because
            // it's defined by the included headers already.
            //
            // All other cases of duplicate identifiers are reported as an error.
            match api {
                Api::Include(_) => {}
                Api::Struct(strct) => {
                    let ident = &strct.ident;
                    if !type_names.insert(ident)
                        && (!cxx.contains(ident)
                            || structs.contains_key(ident)
                            || enums.contains_key(ident))
                    {
                        // If already declared as a struct or enum, or if
                        // colliding with something other than an extern C++
                        // type, then error.
                        duplicate_name(cx, strct, ident);
                    }
                    structs.insert(ident, strct);
                    for field in &strct.fields {
                        visit(&mut all, &field.ty);
                    }
                }
                Api::Enum(enm) => {
                    let ident = &enm.ident;
                    if !type_names.insert(ident)
                        && (!cxx.contains(ident)
                            || structs.contains_key(ident)
                            || enums.contains_key(ident))
                    {
                        // If already declared as a struct or enum, or if
                        // colliding with something other than an extern C++
                        // type, then error.
                        duplicate_name(cx, enm, ident);
                    }
                    enums.insert(ident, enm);
                }
                Api::CxxType(ety) => {
                    let ident = &ety.ident;
                    if !type_names.insert(ident)
                        && (cxx.contains(ident)
                            || !structs.contains_key(ident) && !enums.contains_key(ident))
                    {
                        // If already declared as an extern C++ type, or if
                        // colliding with something which is neither struct nor
                        // enum, then error.
                        duplicate_name(cx, ety, ident);
                    }
                    cxx.insert(ident);
                    if !ety.trusted {
                        untrusted.insert(ident, ety);
                    }
                }
                Api::RustType(ety) => {
                    let ident = &ety.ident;
                    if !type_names.insert(ident) {
                        duplicate_name(cx, ety, ident);
                    }
                    rust.insert(ident);
                }
                Api::CxxFunction(efn) | Api::RustFunction(efn) => {
                    let ident = &efn.ident;
                    if !function_names.insert((&efn.receiver, ident)) {
                        duplicate_name(cx, efn, ident);
                    }
                    for arg in &efn.args {
                        visit(&mut all, &arg.ty);
                    }
                    if let Some(ret) = &efn.ret {
                        visit(&mut all, ret);
                    }
                }
                Api::TypeAlias(alias) => {
                    let ident = &alias.ident;
                    if !type_names.insert(ident) {
                        duplicate_name(cx, alias, ident);
                    }
                    cxx.insert(ident);
                    aliases.insert(ident, alias);
                }
            }
        }

        Types {
            all,
            structs,
            enums,
            cxx,
            rust,
            aliases,
            untrusted,
        }
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

fn duplicate_name(cx: &mut Errors, sp: impl ToTokens, ident: &Ident) {
    let msg = format!("the name `{}` is defined multiple times", ident);
    cx.error(sp, msg);
}
