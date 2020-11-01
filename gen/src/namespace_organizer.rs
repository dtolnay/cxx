use crate::syntax::Api;
use proc_macro2::Ident;
use std::collections::BTreeMap;
use std::iter::FromIterator;

pub struct NamespaceEntries<'a> {
    direct: Vec<&'a Api>,
    nested: BTreeMap<&'a Ident, NamespaceEntries<'a>>,
}

impl<'a> NamespaceEntries<'a> {
    pub fn new(apis: &'a [Api]) -> Self {
        let api_refs = Vec::from_iter(apis);
        Self::sort_by_inner_namespace(api_refs, 0)
    }

    pub fn direct_content(&self) -> &[&'a Api] {
        &self.direct
    }

    pub fn nested_content(&self) -> impl Iterator<Item = (&Ident, &NamespaceEntries)> {
        self.nested.iter().map(|(k, entries)| (*k, entries))
    }

    fn sort_by_inner_namespace(apis: Vec<&'a Api>, depth: usize) -> Self {
        let mut root = NamespaceEntries {
            direct: Vec::new(),
            nested: BTreeMap::new(),
        };

        let mut nested_namespaces = BTreeMap::new();
        for api in apis {
            if let Some(ns) = api.namespace() {
                let first_ns_elem = ns.iter().nth(depth);
                if let Some(first_ns_elem) = first_ns_elem {
                    let list = nested_namespaces.entry(first_ns_elem).or_insert(Vec::new());
                    list.push(api);
                    continue;
                }
            }
            root.direct.push(api);
        }

        for (k, v) in nested_namespaces.into_iter() {
            root.nested
                .insert(k, Self::sort_by_inner_namespace(v, depth + 1));
        }

        root
    }
}

#[cfg(test)]
mod tests {
    use super::NamespaceEntries;
    use crate::syntax::namespace::Namespace;
    use crate::syntax::{Api, Doc, ExternType, Pair};
    use proc_macro2::{Ident, Span};
    use syn::Token;

    #[test]
    fn test_ns_entries_sort() {
        let entries = vec![
            make_api(None, "C"),
            make_api(None, "A"),
            make_api(Some("G"), "E"),
            make_api(Some("D"), "F"),
            make_api(Some("G"), "H"),
            make_api(Some("D::K"), "L"),
            make_api(Some("D::K"), "M"),
            make_api(None, "B"),
            make_api(Some("D"), "I"),
            make_api(Some("D"), "J"),
        ];
        let ns = NamespaceEntries::new(&entries);
        let root_entries = ns.direct_content();
        assert_eq!(root_entries.len(), 3);
        assert_ident(root_entries[0], "C");
        assert_ident(root_entries[1], "A");
        assert_ident(root_entries[2], "B");
        let mut kids = ns.nested_content();
        let (d_id, d_nse) = kids.next().unwrap();
        assert_eq!(d_id.to_string(), "D");
        let (g_id, g_nse) = kids.next().unwrap();
        assert_eq!(g_id.to_string(), "G");
        assert!(kids.next().is_none());
        let d_nse_entries = d_nse.direct_content();
        assert_eq!(d_nse_entries.len(), 3);
        assert_ident(d_nse_entries[0], "F");
        assert_ident(d_nse_entries[1], "I");
        assert_ident(d_nse_entries[2], "J");
        let g_nse_entries = g_nse.direct_content();
        assert_eq!(g_nse_entries.len(), 2);
        assert_ident(g_nse_entries[0], "E");
        assert_ident(g_nse_entries[1], "H");
        let mut g_kids = g_nse.nested_content();
        assert!(g_kids.next().is_none());
        let mut d_kids = d_nse.nested_content();
        let (k_id, k_nse) = d_kids.next().unwrap();
        assert_eq!(k_id.to_string(), "K");
        let k_nse_entries = k_nse.direct_content();
        assert_eq!(k_nse_entries.len(), 2);
        assert_ident(k_nse_entries[0], "L");
        assert_ident(k_nse_entries[1], "M");
    }

    fn assert_ident(api: &Api, expected: &str) {
        if let Api::CxxType(cxx_type) = api {
            assert_eq!(cxx_type.ident.cxx.ident.to_string(), expected);
        } else {
            unreachable!()
        }
    }

    fn make_api(ns: Option<&str>, ident: &str) -> Api {
        let ns = match ns {
            Some(st) => Namespace::from_str(st),
            None => Namespace::none(),
        };
        let ident = Pair::new(ns, Ident::new(ident, Span::call_site()));
        Api::CxxType(ExternType {
            doc: Doc::new(),
            type_token: Token![type](Span::call_site()),
            ident,
            semi_token: Token![;](Span::call_site()),
            trusted: true,
        })
    }
}
