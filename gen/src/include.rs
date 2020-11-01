use crate::gen::out::Content;
use crate::syntax::{self, IncludeKind};
use std::ops::{Deref, DerefMut};

/// The complete contents of the "rust/cxx.h" header.
pub static HEADER: &str = include_str!("include/cxx.h");

/// A header to #include.
///
/// The cxxbridge tool does not parse or even require the given paths to exist;
/// they simply go into the generated C++ code as #include lines.
#[derive(Clone, PartialEq, Debug)]
pub struct Include {
    /// The header's path, not including the enclosing quotation marks or angle
    /// brackets.
    pub path: String,
    /// Whether to emit `#include "path"` or `#include <path>`.
    pub kind: IncludeKind,
}

#[derive(Default, PartialEq)]
pub struct Includes {
    pub custom: Vec<Include>,
    pub array: bool,
    pub cstddef: bool,
    pub cstdint: bool,
    pub cstring: bool,
    pub exception: bool,
    pub memory: bool,
    pub new: bool,
    pub string: bool,
    pub type_traits: bool,
    pub utility: bool,
    pub vector: bool,
    pub basetsd: bool,
    pub content: Content,
}

impl Includes {
    pub fn new() -> Self {
        Includes::default()
    }

    pub fn insert(&mut self, include: impl Into<Include>) {
        self.custom.push(include.into());
    }
}

impl<'a> Extend<&'a Include> for Includes {
    fn extend<I: IntoIterator<Item = &'a Include>>(&mut self, iter: I) {
        self.custom.extend(iter.into_iter().cloned());
    }
}

impl<'a> From<&'a syntax::Include> for Include {
    fn from(include: &syntax::Include) -> Self {
        Include {
            path: include.path.clone(),
            kind: include.kind,
        }
    }
}

impl Deref for Includes {
    type Target = Content;

    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

impl DerefMut for Includes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.content
    }
}
