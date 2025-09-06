use crate::syntax::cfg::ComputedCfg;
use crate::syntax::map::{OrderedMap, UnorderedMap};
use crate::syntax::set::UnorderedSet;
use crate::syntax::{Api, Enum, NamedType, Receiver, Ref, Struct, Type, TypeAlias};
use proc_macro2::Ident;

#[allow(dead_code)] // only used by cxxbridge-macro, not cxx-build
pub(crate) enum UnpinReason<'a> {
    Receiver(&'a Receiver),
    Ref(&'a Ref),
    Slice,
}

pub(crate) fn required_unpin_reasons<'a>(
    apis: &'a [Api],
    all: &OrderedMap<&'a Type, ComputedCfg>,
    structs: &UnorderedMap<&'a Ident, &'a Struct>,
    enums: &UnorderedMap<&'a Ident, &'a Enum>,
    cxx: &UnorderedSet<&'a Ident>,
    aliases: &UnorderedMap<&'a Ident, &'a TypeAlias>,
) -> UnorderedMap<&'a Ident, UnpinReason<'a>> {
    let mut reasons = UnorderedMap::new();

    let is_extern_type_alias = |ty: &NamedType| -> bool {
        cxx.contains(&ty.rust)
            && !structs.contains_key(&ty.rust)
            && !enums.contains_key(&ty.rust)
            && aliases.contains_key(&ty.rust)
    };

    for (ty, _cfgs) in all {
        if let Type::SliceRef(ty) = ty {
            if let Type::Ident(inner) = &ty.inner {
                if is_extern_type_alias(inner) {
                    reasons.insert(&inner.rust, UnpinReason::Slice);
                }
            }
        }
    }

    for api in apis {
        if let Api::CxxFunction(efn) | Api::RustFunction(efn) = api {
            if let Some(receiver) = efn.receiver() {
                if receiver.mutable && !receiver.pinned && is_extern_type_alias(&receiver.ty) {
                    reasons.insert(&receiver.ty.rust, UnpinReason::Receiver(receiver));
                }
            }
        }
    }

    for (ty, _cfg) in all {
        if let Type::Ref(ty) = ty {
            if let Type::Ident(inner) = &ty.inner {
                if ty.mutable && !ty.pinned && is_extern_type_alias(inner) {
                    reasons.insert(&inner.rust, UnpinReason::Ref(ty));
                }
            }
        }
    }

    reasons
}
