use crate::syntax::set::OrderedSet as Set;
use crate::syntax::{Api, Enum, ExternFn, ExternType, RustName, Struct, Type};
use proc_macro2::Ident;
use std::collections::BTreeMap as Map;
use std::fmt::Display;

#[derive(Copy, Clone)]
pub enum TrivialReason<'a> {
    StructField(&'a Struct),
    FunctionArgument(&'a ExternFn),
    FunctionReturn(&'a ExternFn),
    BoxTarget,
    VecElement,
    UnpinnedMutArg(&'a ExternFn),
}

pub fn required_trivial_reasons<'a>(
    apis: &'a [Api],
    all: &Set<&'a Type>,
    structs: &Map<&'a Ident, &'a Struct>,
    enums: &Map<&'a Ident, &'a Enum>,
    cxx: &Set<&'a Ident>,
) -> Map<&'a Ident, Vec<TrivialReason<'a>>> {
    let mut required_trivial = Map::new();

    let mut insist_extern_types_are_trivial = |ident: &'a RustName, reason| {
        if cxx.contains(&ident.rust)
            && !structs.contains_key(&ident.rust)
            && !enums.contains_key(&ident.rust)
        {
            required_trivial
                .entry(&ident.rust)
                .or_insert_with(Vec::new)
                .push(reason);
        }
    };

    for api in apis {
        match api {
            Api::Struct(strct) => {
                for field in &strct.fields {
                    if let Type::Ident(ident) = &field.ty {
                        let reason = TrivialReason::StructField(strct);
                        insist_extern_types_are_trivial(ident, reason);
                    }
                }
            }
            Api::CxxFunction(efn) | Api::RustFunction(efn) => {
                if let Some(receiver) = &efn.receiver {
                    if receiver.mutable && !receiver.pinned {
                        let reason = TrivialReason::UnpinnedMutArg(efn);
                        insist_extern_types_are_trivial(&receiver.ty, reason);
                    }
                }
                for arg in &efn.args {
                    match &arg.ty {
                        Type::Ident(ident) => {
                            let reason = TrivialReason::FunctionArgument(efn);
                            insist_extern_types_are_trivial(ident, reason);
                        }
                        Type::Ref(ty) => {
                            if ty.mutable && !ty.pinned {
                                if let Type::Ident(ident) = &ty.inner {
                                    let reason = TrivialReason::UnpinnedMutArg(efn);
                                    insist_extern_types_are_trivial(ident, reason);
                                }
                            }
                        }
                        _ => {}
                    }
                }
                if let Some(ret) = &efn.ret {
                    if let Type::Ident(ident) = &ret {
                        let reason = TrivialReason::FunctionReturn(efn);
                        insist_extern_types_are_trivial(ident, reason);
                    }
                }
            }
            _ => {}
        }
    }

    for ty in all {
        match ty {
            Type::RustBox(ty) => {
                if let Type::Ident(ident) = &ty.inner {
                    let reason = TrivialReason::BoxTarget;
                    insist_extern_types_are_trivial(ident, reason);
                }
            }
            Type::RustVec(ty) => {
                if let Type::Ident(ident) = &ty.inner {
                    let reason = TrivialReason::VecElement;
                    insist_extern_types_are_trivial(ident, reason);
                }
            }
            _ => {}
        }
    }

    required_trivial
}

impl<'a> TrivialReason<'a> {
    pub fn describe_in_context(&self, ety: &ExternType) -> String {
        match self {
            TrivialReason::BoxTarget => format!("Box<{}>", ety.name.rust),
            TrivialReason::VecElement => format!("a vector element in Vec<{}>", ety.name.rust),
            _ => self.to_string(),
        }
    }
}

impl<'a> Display for TrivialReason<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrivialReason::StructField(strct) => write!(f, "a field of `{}`", strct.name.rust),
            TrivialReason::FunctionArgument(efn) => write!(f, "an argument of `{}`", efn.name.rust),
            TrivialReason::FunctionReturn(efn) => {
                write!(f, "a return value of `{}`", efn.name.rust)
            }
            TrivialReason::BoxTarget => write!(f, "in a Box<...>"),
            TrivialReason::VecElement => write!(f, "a Vec<...> element"),
            TrivialReason::UnpinnedMutArg(efn) => write!(
                f,
                "a non-pinned mutable reference argument of {}",
                efn.name.rust
            ),
        }
    }
}
