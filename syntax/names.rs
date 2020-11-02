use crate::syntax::{Namespace, Pair, ResolvableName, Symbol, Types};
use proc_macro2::{Ident, Span};
use std::iter;
use syn::Token;

impl Pair {
    // Use this constructor when the item can't have a different name in Rust
    // and C++.
    pub fn new(namespace: Namespace, ident: Ident) -> Self {
        Self {
            namespace,
            cxx: ident.clone(),
            rust: ident,
        }
    }

    // Use this constructor when attributes such as #[rust_name] can be used to
    // potentially give a different name in Rust vs C++.
    pub fn new_from_differing_names(
        namespace: Namespace,
        cxx_ident: Ident,
        rust_ident: Ident,
    ) -> Self {
        Self {
            namespace,
            cxx: cxx_ident,
            rust: rust_ident,
        }
    }

    pub fn to_symbol(&self) -> Symbol {
        Symbol::from_idents(self.iter_all_segments())
    }

    pub fn to_fully_qualified(&self) -> String {
        format!("::{}", self.join("::"))
    }

    fn iter_all_segments(&self) -> impl Iterator<Item = &Ident> {
        self.namespace.iter().chain(iter::once(&self.cxx))
    }

    fn join(&self, sep: &str) -> String {
        self.iter_all_segments()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join(sep)
    }
}

impl ResolvableName {
    pub fn new(ident: Ident) -> Self {
        Self { rust: ident }
    }

    pub fn make_self(span: Span) -> Self {
        Self {
            rust: Token![Self](span).into(),
        }
    }

    pub fn is_self(&self) -> bool {
        self.rust == "Self"
    }

    pub fn span(&self) -> Span {
        self.rust.span()
    }

    pub fn to_symbol(&self, types: &Types) -> Symbol {
        types.resolve(self).to_symbol()
    }
}
