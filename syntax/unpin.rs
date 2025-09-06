use crate::syntax::cfg::ComputedCfg;
use crate::syntax::map::{OrderedMap, UnorderedMap};
use crate::syntax::set::UnorderedSet;
use crate::syntax::{Api, Enum, Struct, Type, TypeAlias};
use proc_macro2::Ident;

pub(crate) fn required_unpin_aliases<'a>(
    apis: &'a [Api],
    all: &OrderedMap<&'a Type, ComputedCfg>,
    structs: &UnorderedMap<&'a Ident, &'a Struct>,
    enums: &UnorderedMap<&'a Ident, &'a Enum>,
    cxx: &UnorderedSet<&'a Ident>,
    aliases: &UnorderedMap<&'a Ident, &'a TypeAlias>,
) -> UnorderedSet<&'a Ident> {
    let mut required_unpin = UnorderedSet::new();

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
                    required_unpin.insert(&receiver.ty.rust);
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
                    required_unpin.insert(&inner.rust);
                }
            }
        }
    }

    required_unpin
}
