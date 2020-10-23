use crate::syntax::Api;
use proc_macro2::Ident;
use std::collections::HashMap;

pub(crate) struct NamespaceEntries<'a> {
    pub(crate) entries: Vec<&'a Api>,
    pub(crate) children: HashMap<&'a Ident, NamespaceEntries<'a>>,
}

pub(crate) fn sort_by_namespace(apis: &[Api]) -> NamespaceEntries {
    let api_refs = apis.iter().collect::<Vec<_>>();
    sort_by_inner_namespace(api_refs, 0)
}

fn sort_by_inner_namespace(apis: Vec<&Api>, depth: usize) -> NamespaceEntries {
    let mut root = NamespaceEntries {
        entries: Vec::new(),
        children: HashMap::new(),
    };

    let mut kids_by_child_ns = HashMap::new();
    for api in apis {
        if let Some(ns) = api.get_namespace() {
            let first_ns_elem = ns.iter().skip(depth).next();
            if let Some(first_ns_elem) = first_ns_elem {
                let list = kids_by_child_ns.entry(first_ns_elem).or_insert(Vec::new());
                list.push(api);
                continue;
            }
        }
        root.entries.push(api);
    }

    for (k, v) in kids_by_child_ns.into_iter() {
        root.children
            .insert(k, sort_by_inner_namespace(v, depth + 1));
    }

    root
}
