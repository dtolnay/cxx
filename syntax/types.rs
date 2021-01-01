use crate::syntax::improper::ImproperCtype;
use crate::syntax::instantiate::ImplKey;
use crate::syntax::map::UnorderedMap;
use crate::syntax::report::Errors;
use crate::syntax::set::{OrderedSet as Set, UnorderedSet};
use crate::syntax::trivial::{self, TrivialReason};
use crate::syntax::{
    toposort, Api, Enum, ExternType, Impl, Pair, RustName, Struct, Type, TypeAlias,
};
use proc_macro2::Ident;
use quote::ToTokens;

pub struct Types<'a> {
    pub all: Set<&'a Type>,
    pub structs: UnorderedMap<&'a Ident, &'a Struct>,
    pub enums: UnorderedMap<&'a Ident, &'a Enum>,
    pub cxx: UnorderedSet<&'a Ident>,
    pub rust: UnorderedSet<&'a Ident>,
    pub aliases: UnorderedMap<&'a Ident, &'a TypeAlias>,
    pub untrusted: UnorderedMap<&'a Ident, &'a ExternType>,
    pub required_trivial: UnorderedMap<&'a Ident, Vec<TrivialReason<'a>>>,
    pub explicit_impls: UnorderedMap<ImplKey<'a>, &'a Impl>,
    pub resolutions: UnorderedMap<&'a Ident, &'a Pair>,
    pub struct_improper_ctypes: UnorderedSet<&'a Ident>,
    pub toposorted_structs: Vec<&'a Struct>,
}

impl<'a> Types<'a> {
    pub fn collect(cx: &mut Errors, apis: &'a [Api]) -> Self {
        let mut all = Set::new();
        let mut structs = UnorderedMap::new();
        let mut enums = UnorderedMap::new();
        let mut cxx = UnorderedSet::new();
        let mut rust = UnorderedSet::new();
        let mut aliases = UnorderedMap::new();
        let mut untrusted = UnorderedMap::new();
        let mut explicit_impls = UnorderedMap::new();
        let mut resolutions = UnorderedMap::new();
        let struct_improper_ctypes = UnorderedSet::new();
        let toposorted_structs = Vec::new();

        fn visit<'a>(all: &mut Set<&'a Type>, ty: &'a Type) {
            all.insert(ty);
            match ty {
                Type::Ident(_) | Type::Str(_) | Type::Void(_) => {}
                Type::RustBox(ty)
                | Type::UniquePtr(ty)
                | Type::SharedPtr(ty)
                | Type::WeakPtr(ty)
                | Type::CxxVector(ty)
                | Type::RustVec(ty) => visit(all, &ty.inner),
                Type::Ref(r) => visit(all, &r.inner),
                Type::Array(a) => visit(all, &a.inner),
                Type::SliceRef(s) => visit(all, &s.inner),
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
                    all.insert(&enm.repr_type);
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
                    if let Some(key) = imp.ty.impl_key() {
                        explicit_impls.insert(key, imp);
                    }
                }
            }
        }

        // All these APIs may contain types passed by value. We need to ensure
        // we check that this is permissible. We do this _after_ scanning all
        // the APIs above, in case some function or struct references a type
        // which is declared subsequently.
        let required_trivial =
            trivial::required_trivial_reasons(apis, &all, &structs, &enums, &cxx);

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

        let mut unresolved_structs = types.structs.keys();
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
            Type::RustBox(_) | Type::UniquePtr(_) => false,
            Type::Array(_) => true,
            _ => !self.is_guaranteed_pod(ty),
        }
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

    pub fn resolve(&self, ident: &RustName) -> &Pair {
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

fn duplicate_name(cx: &mut Errors, sp: impl ToTokens, ident: &Ident) {
    let msg = format!("the name `{}` is defined multiple times", ident);
    cx.error(sp, msg);
}
