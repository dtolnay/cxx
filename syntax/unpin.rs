use crate::syntax::cfg::ComputedCfg;
use crate::syntax::map::{OrderedMap, UnorderedMap};
use crate::syntax::set::UnorderedSet;
use crate::syntax::{Api, Enum, Receiver, Ref, Struct, Type, TypeAlias};
use proc_macro2::Ident;

#[allow(dead_code)] // only used by cxxbridge-macro, not cxx-build
pub(crate) enum UnpinReason<'a> {
    Receiver(&'a Receiver),
    Ref(&'a Ref),
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

    for api in apis {
        if let Api::CxxFunction(efn) | Api::RustFunction(efn) = api {
            if let Some(receiver) = efn.receiver() {
                if receiver.mutable
                    && !receiver.pinned
                    && cxx.contains(&receiver.ty.rust)
                    && !structs.contains_key(&receiver.ty.rust)
                    && !enums.contains_key(&receiver.ty.rust)
                    && aliases.contains_key(&receiver.ty.rust)
                {
                    reasons.insert(&receiver.ty.rust, UnpinReason::Receiver(receiver));
                }
            }
        }
    }

    for (ty, _cfg) in all {
        if let Type::Ref(ty) = ty {
            if let Type::Ident(inner) = &ty.inner {
                if ty.mutable
                    && !ty.pinned
                    && cxx.contains(&inner.rust)
                    && !structs.contains_key(&inner.rust)
                    && !enums.contains_key(&inner.rust)
                    && aliases.contains_key(&inner.rust)
                {
                    reasons.insert(&inner.rust, UnpinReason::Ref(ty));
                }
            }
        }
    }

    reasons
}
