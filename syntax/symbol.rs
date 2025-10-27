use crate::syntax::namespace::Namespace;
use crate::syntax::{ForeignName, Pair};
use proc_macro2::{Ident, TokenStream};
use quote::{IdentFragment, ToTokens};
use std::fmt::{self, Display, Write};

// A mangled symbol consisting of segments separated by '$'.
//
// Segments are expected to only contain characters that are valid inside
// both C++ and Rust identifiers (
// [XID_Start or XID_Continue](https://doc.rust-lang.org/reference/identifiers.html),
// but not a `$` sign).
//
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

impl IdentFragment for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Need to escape non-identifier-characters
        // (`$` is the only such character allowed in `Symbol`s).
        //
        // The escaping scheme needs to be
        // [an injection](https://en.wikipedia.org/wiki/Injective_function).
        // This means that we also need to escape the escape character `_`.
        for c in self.0.chars() {
            match c {
                '_' => f.write_str("_u")?,
                '$' => f.write_str("_d")?,
                c => {
                    // TODO: Assert that `c` is XID_Start or XID_Continue?
                    f.write_fmt(format_args!("{}", c))?;
                }
            }
        }
        Ok(())
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

    #[allow(dead_code)] // only used by cxx-gen, not cxxbridge-macro
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
        // TODO: support C++ names containing whitespace (`unsigned int`) or
        // non-alphanumeric characters (`operator++`).
        self.to_string().write(symbol);
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
