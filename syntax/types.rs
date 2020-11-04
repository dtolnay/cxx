use crate::syntax::atom::Atom::{self, *};
use crate::syntax::improper::ImproperCtype;
use crate::syntax::report::Errors;
use crate::syntax::set::{OrderedSet as Set, UnorderedSet};
use crate::syntax::{
    toposort, Api, Derive, Enum, ExternFn, ExternType, Impl, Pair, ResolvableName, Struct, Type,
    TypeAlias,
};
use proc_macro2::Ident;
use quote::ToTokens;
use std::collections::BTreeMap as Map;

pub struct Types<'a> {
    pub all: Set<&'a Type>,
    pub structs: Map<&'a Ident, &'a Struct>,
    pub enums: Map<&'a Ident, &'a Enum>,
    pub cxx: Set<&'a Ident>,
    pub rust: Set<&'a Ident>,
    pub aliases: Map<&'a Ident, &'a TypeAlias>,
    pub untrusted: Map<&'a Ident, &'a ExternType>,
    pub required_trivial: Map<&'a Ident, TrivialReason<'a>>,
    pub explicit_impls: Set<&'a Impl>,
    pub resolutions: Map<&'a Ident, &'a Pair>,
    pub struct_improper_ctypes: UnorderedSet<&'a Ident>,
    pub toposorted_structs: Vec<&'a Struct>,
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
        let mut explicit_impls = Set::new();
        let mut resolutions = Map::new();
        let struct_improper_ctypes = UnorderedSet::new();
        let toposorted_structs = Vec::new();

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

        let mut add_resolution = |pair: &'a Pair| {
            resolutions.insert(&pair.rust, pair);
        };

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
                    let ident = &strct.name.rust;
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
                    structs.insert(&strct.name.rust, strct);
                    for field in &strct.fields {
                        visit(&mut all, &field.ty);
                    }
                    add_resolution(&strct.name);
                }
                Api::Enum(enm) => {
                    let ident = &enm.name.rust;
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
                    add_resolution(&enm.name);
                }
                Api::CxxType(ety) => {
                    let ident = &ety.name.rust;
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
                    add_resolution(&ety.name);
                }
                Api::RustType(ety) => {
                    let ident = &ety.name.rust;
                    if !type_names.insert(ident) {
                        duplicate_name(cx, ety, ident);
                    }
                    rust.insert(ident);
                    add_resolution(&ety.name);
                }
                Api::CxxFunction(efn) | Api::RustFunction(efn) => {
                    // Note: duplication of the C++ name is fine because C++ has
                    // function overloading.
                    if !function_names.insert((&efn.receiver, &efn.name.rust)) {
                        duplicate_name(cx, efn, &efn.name.rust);
                    }
                    for arg in &efn.args {
                        visit(&mut all, &arg.ty);
                    }
                    if let Some(ret) = &efn.ret {
                        visit(&mut all, ret);
                    }
                }
                Api::TypeAlias(alias) => {
                    let ident = &alias.name.rust;
                    if !type_names.insert(ident) {
                        duplicate_name(cx, alias, ident);
                    }
                    cxx.insert(ident);
                    aliases.insert(ident, alias);
                    add_resolution(&alias.name);
                }
                Api::Impl(imp) => {
                    visit(&mut all, &imp.ty);
                    explicit_impls.insert(imp);
                }
            }
        }

        // All these APIs may contain types passed by value. We need to ensure
        // we check that this is permissible. We do this _after_ scanning all
        // the APIs above, in case some function or struct references a type
        // which is declared subsequently.
        let mut required_trivial = Map::new();
        let mut insist_alias_types_are_trivial = |ty: &'a Type, reason| {
            if let Type::Ident(ident) = ty {
                if cxx.contains(&ident.rust) {
                    required_trivial.entry(&ident.rust).or_insert(reason);
                }
            }
        };
        for api in apis {
            match api {
                Api::Struct(strct) => {
                    let reason = TrivialReason::StructField(strct);
                    for field in &strct.fields {
                        insist_alias_types_are_trivial(&field.ty, reason);
                    }
                }
                Api::CxxFunction(efn) | Api::RustFunction(efn) => {
                    let reason = TrivialReason::FunctionArgument(efn);
                    for arg in &efn.args {
                        insist_alias_types_are_trivial(&arg.ty, reason);
                    }
                    if let Some(ret) = &efn.ret {
                        let reason = TrivialReason::FunctionReturn(efn);
                        insist_alias_types_are_trivial(&ret, reason);
                    }
                }
                _ => {}
            }
        }

        let mut types = Types {
            all,
            structs,
            enums,
            cxx,
            rust,
            aliases,
            untrusted,
            required_trivial,
            explicit_impls,
            resolutions,
            struct_improper_ctypes,
            toposorted_structs,
        };

        types.toposorted_structs = toposort::sort(cx, apis, &types);

        let mut unresolved_structs: Vec<&Ident> = types.structs.keys().copied().collect();
        let mut new_information = true;
        while new_information {
            new_information = false;
            unresolved_structs.retain(|ident| {
                let mut retain = false;
                for var in &types.structs[ident].fields {
                    if match types.determine_improper_ctype(&var.ty) {
                        ImproperCtype::Depends(inner) => {
                            retain = true;
                            types.struct_improper_ctypes.contains(inner)
                        }
                        ImproperCtype::Definite(improper) => improper,
                    } {
                        types.struct_improper_ctypes.insert(ident);
                        new_information = true;
                        return false;
                    }
                }
                // If all fields definite false, remove from unresolved_structs.
                retain
            });
        }

        types
    }

    pub fn needs_indirect_abi(&self, ty: &Type) -> bool {
        match ty {
            Type::Ident(ident) => {
                if let Some(strct) = self.structs.get(&ident.rust) {
                    !self.is_pod(strct)
                } else {
                    Atom::from(&ident.rust) == Some(RustString)
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

    // Types that trigger rustc's default #[warn(improper_ctypes)] lint, even if
    // they may be otherwise unproblematic to mention in an extern signature.
    // For example in a signature like `extern "C" fn(*const String)`, rustc
    // refuses to believe that C could know how to supply us with a pointer to a
    // Rust String, even though C could easily have obtained that pointer
    // legitimately from a Rust call.
    pub fn is_considered_improper_ctype(&self, ty: &Type) -> bool {
        match self.determine_improper_ctype(ty) {
            ImproperCtype::Definite(improper) => improper,
            ImproperCtype::Depends(ident) => self.struct_improper_ctypes.contains(ident),
        }
    }

    pub fn resolve(&self, ident: &ResolvableName) -> &Pair {
        self.resolutions
            .get(&ident.rust)
            .expect("Unable to resolve type")
    }
}

impl<'t, 'a> IntoIterator for &'t Types<'a> {
    type Item = &'a Type;
    type IntoIter = crate::syntax::set::Iter<'t, 'a, Type>;
    fn into_iter(self) -> Self::IntoIter {
        self.all.into_iter()
    }
}

#[derive(Copy, Clone)]
pub enum TrivialReason<'a> {
    StructField(&'a Struct),
    FunctionArgument(&'a ExternFn),
    FunctionReturn(&'a ExternFn),
}

fn duplicate_name(cx: &mut Errors, sp: impl ToTokens, ident: &Ident) {
    let msg = format!("the name `{}` is defined multiple times", ident);
    cx.error(sp, msg);
}
