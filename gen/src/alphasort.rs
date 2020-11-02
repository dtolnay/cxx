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
        sort_by_inner_namespace(api_refs, 0)
    }

    pub fn direct_content(&self) -> &[&'a Api] {
        &self.direct
    }

    pub fn nested_content(&self) -> impl Iterator<Item = (&'a Ident, &NamespaceEntries<'a>)> {
        self.nested.iter().map(|(k, entries)| (*k, entries))
    }
}

fn sort_by_inner_namespace(apis: Vec<&Api>, depth: usize) -> NamespaceEntries {
    let mut direct = Vec::new();
    let mut nested_namespaces = BTreeMap::new();
    for api in &apis {
        if let Some(first_ns_elem) = api.namespace().and_then(|ns| ns.iter().nth(depth)) {
            nested_namespaces
                .entry(first_ns_elem)
                .or_insert_with(Vec::new)
                .push(*api);
            continue;
        }
        direct.push(*api);
    }

    let nested = nested_namespaces
        .into_iter()
        .map(|(k, apis)| (k, sort_by_inner_namespace(apis, depth + 1)))
        .collect();

    NamespaceEntries { direct, nested }
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
        let apis = &[
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

        let root = NamespaceEntries::new(apis);

        // ::
        let root_direct = root.direct_content();
        assert_eq!(root_direct.len(), 3);
        assert_ident(root_direct[0], "C");
        assert_ident(root_direct[1], "A");
        assert_ident(root_direct[2], "B");

        let mut root_nested = root.nested_content();
        let (id, d) = root_nested.next().unwrap();
        assert_eq!(id, "D");
        let (id, g) = root_nested.next().unwrap();
        assert_eq!(id, "G");
        assert!(root_nested.next().is_none());

        // ::D
        let d_direct = d.direct_content();
        assert_eq!(d_direct.len(), 3);
        assert_ident(d_direct[0], "F");
        assert_ident(d_direct[1], "I");
        assert_ident(d_direct[2], "J");

        let mut d_nested = d.nested_content();
        let (id, k) = d_nested.next().unwrap();
        assert_eq!(id, "K");

        // ::D::K
        let k_direct = k.direct_content();
        assert_eq!(k_direct.len(), 2);
        assert_ident(k_direct[0], "L");
        assert_ident(k_direct[1], "M");

        // ::G
        let g_direct = g.direct_content();
        assert_eq!(g_direct.len(), 2);
        assert_ident(g_direct[0], "E");
        assert_ident(g_direct[1], "H");

        let mut g_nested = g.nested_content();
        assert!(g_nested.next().is_none());
    }

    fn assert_ident(api: &Api, expected: &str) {
        if let Api::CxxType(cxx_type) = api {
            assert_eq!(cxx_type.ident.cxx.ident, expected);
        } else {
            unreachable!()
        }
    }

    fn make_api(ns: Option<&str>, ident: &str) -> Api {
        let ns = ns.map_or(Namespace::ROOT, |ns| syn::parse_str(ns).unwrap());
        Api::CxxType(ExternType {
            doc: Doc::new(),
            type_token: Token![type](Span::call_site()),
            ident: Pair::new(ns, Ident::new(ident, Span::call_site())),
            semi_token: Token![;](Span::call_site()),
            trusted: false,
        })
    }
}
