use crate::syntax::Api;
use proc_macro2::Ident;
use std::collections::BTreeMap;

pub struct NamespaceEntries<'a> {
    entries: Vec<&'a Api>,
    children: BTreeMap<&'a Ident, NamespaceEntries<'a>>,
}

impl<'a> NamespaceEntries<'a> {
    pub fn new(apis: &'a [Api]) -> Self {
        let api_refs = apis.iter().collect::<Vec<_>>();
        Self::sort_by_inner_namespace(api_refs, 0)
    }

    pub fn entries(&self) -> &[&'a Api] {
        &self.entries
    }

    pub fn children(&self) -> impl Iterator<Item = (&&Ident, &NamespaceEntries)> {
        self.children.iter()
    }

    fn sort_by_inner_namespace(apis: Vec<&'a Api>, depth: usize) -> Self {
        let mut root = NamespaceEntries {
            entries: Vec::new(),
            children: BTreeMap::new(),
        };

        let mut kids_by_child_ns = BTreeMap::new();
        for api in apis {
            if let Some(ns) = api.get_namespace() {
                let first_ns_elem = ns.iter().nth(depth);
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
                .insert(k, Self::sort_by_inner_namespace(v, depth + 1));
        }

        root
    }
}
