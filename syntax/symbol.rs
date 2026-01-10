use crate::syntax::namespace::Namespace;
use crate::syntax::{ForeignName, Pair};
use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;
use std::fmt::{self, Display, Write};

// A mangled symbol consisting of segments separated by '$'.
// Example: cxxbridge1$string$new
#[derive(Eq, Hash, PartialEq)]
pub(crate) struct Symbol(String);

impl Display for Symbol {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.0, formatter)
    }
}

impl ToTokens for Symbol {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        ToTokens::to_tokens(&self.0, tokens);
    }
}

impl Symbol {
    fn push(&mut self, segment: &dyn Display) {
        let len_before = self.0.len();
        if !self.0.is_empty() {
            self.0.push('$');
        }
        self.0.write_fmt(format_args!("{}", segment)).unwrap();
        assert!(self.0.len() > len_before);
    }

    pub(crate) fn from_idents<'a>(it: impl Iterator<Item = &'a dyn Segment>) -> Self {
        let mut symbol = Symbol(String::new());
        for segment in it {
            segment.write(&mut symbol);
        }
        assert!(!symbol.0.is_empty());
        symbol
    }

    #[cfg_attr(proc_macro, expect(dead_code))]
    pub(crate) fn contains(&self, ch: char) -> bool {
        self.0.contains(ch)
    }
}

pub(crate) trait Segment {
    fn write(&self, symbol: &mut Symbol);
}

impl Segment for str {
    fn write(&self, symbol: &mut Symbol) {
        symbol.push(&self);
    }
}

impl Segment for usize {
    fn write(&self, symbol: &mut Symbol) {
        symbol.push(&self);
    }
}

impl Segment for Ident {
    fn write(&self, symbol: &mut Symbol) {
        symbol.push(&self);
    }
}

impl Segment for Symbol {
    fn write(&self, symbol: &mut Symbol) {
        symbol.push(&self);
    }
}

impl Segment for Namespace {
    fn write(&self, symbol: &mut Symbol) {
        for segment in self {
            symbol.push(segment);
        }
    }
}

impl Segment for Pair {
    fn write(&self, symbol: &mut Symbol) {
        self.namespace.write(symbol);
        self.cxx.write(symbol);
    }
}

impl Segment for ForeignName {
    fn write(&self, symbol: &mut Symbol) {
        /// Escapes arbitrary C++ name (e.g. `operator==`) into a String
        /// that is a valid C identifier.  It is important that this is an
        /// [injective function](https://en.wikipedia.org/wiki/Injective_function)
        /// (i.e. distinct `name`s need to map to distinct results).
        fn escape(name: &str) -> String {
            let mut result = String::with_capacity(name.len());
            for (index, ch) in name.chars().enumerate() {
                if ch == '_' {
                    write!(&mut result, "_u").unwrap();
                    continue;
                }

                let should_escape = if index == 0 {
                    !unicode_ident::is_xid_start(ch)
                } else {
                    !unicode_ident::is_xid_continue(ch)
                };
                if should_escape {
                    write!(&mut result, "_{:x}h", ch as u32).unwrap();
                    continue;
                }

                write!(&mut result, "{ch}").unwrap();
            }
            result
        }

        escape(self.as_str()).write(symbol);
    }
}

impl<T> Segment for &'_ T
where
    T: ?Sized + Segment + Display,
{
    fn write(&self, symbol: &mut Symbol) {
        (**self).write(symbol);
    }
}

pub(crate) fn join(segments: &[&dyn Segment]) -> Symbol {
    let mut symbol = Symbol(String::new());
    for segment in segments {
        segment.write(&mut symbol);
    }
    assert!(!symbol.0.is_empty());
    symbol
}

#[cfg(test)]
mod test {
    use super::join;
    use crate::syntax::ForeignName;
    use proc_macro2::Span;

    #[test]
    fn test_impl_segment_for_foreign_name() {
        fn t(foreign_name_str: &str, expected_symbol_str: &str) {
            let foreign_name = ForeignName::parse(foreign_name_str, Span::call_site()).unwrap();
            let symbol = join(&[&foreign_name]);
            let actual_symbol_str = symbol.to_string();
            assert_eq!(
                actual_symbol_str, expected_symbol_str,
                "Expecting `{foreign_name_str}` to mangle as `{expected_symbol_str}` \
                 but got `{actual_symbol_str}` instead.",
            );
        }

        t("foo", "foo");

        // Escaping of non-identifier characters like `=`.
        t("operator==", "operator_3dh_3dh");

        // Feeble attempt of testing injectivity
        // (need to escape `_` to avoid a conflict with result of the previous test).
        t("operator_3dh_3dh", "operator_u3dh_u3dh");
    }
}
