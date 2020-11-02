use crate::gen::out::{Content, OutFile};
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
pub struct Includes<'a> {
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
    pub content: Content<'a>,
}

impl<'a> Includes<'a> {
    pub fn new() -> Self {
        Includes::default()
    }

    pub fn insert(&mut self, include: impl Into<Include>) {
        self.custom.push(include.into());
    }
}

pub(super) fn write(out: &mut OutFile) {
    let header = out.header;
    let include = &mut out.include;
    let out = &mut include.content;

    if header {
        writeln!(out, "#pragma once");
    }

    for include in &include.custom {
        match include.kind {
            IncludeKind::Quoted => {
                writeln!(out, "#include \"{}\"", include.path.escape_default());
            }
            IncludeKind::Bracketed => {
                writeln!(out, "#include <{}>", include.path);
            }
        }
    }

    if include.array {
        writeln!(out, "#include <array>");
    }
    if include.cstddef {
        writeln!(out, "#include <cstddef>");
    }
    if include.cstdint {
        writeln!(out, "#include <cstdint>");
    }
    if include.cstring {
        writeln!(out, "#include <cstring>");
    }
    if include.exception {
        writeln!(out, "#include <exception>");
    }
    if include.memory {
        writeln!(out, "#include <memory>");
    }
    if include.new {
        writeln!(out, "#include <new>");
    }
    if include.string {
        writeln!(out, "#include <string>");
    }
    if include.type_traits {
        writeln!(out, "#include <type_traits>");
    }
    if include.utility {
        writeln!(out, "#include <utility>");
    }
    if include.vector {
        writeln!(out, "#include <vector>");
    }
    if include.basetsd {
        writeln!(out, "#if defined(_WIN32)");
        writeln!(out, "#include <basetsd.h>");
        writeln!(out, "#endif");
    }
}

impl<'i, 'a> Extend<&'i Include> for Includes<'a> {
    fn extend<I: IntoIterator<Item = &'i Include>>(&mut self, iter: I) {
        self.custom.extend(iter.into_iter().cloned());
    }
}

impl<'i> From<&'i syntax::Include> for Include {
    fn from(include: &syntax::Include) -> Self {
        Include {
            path: include.path.clone(),
            kind: include.kind,
        }
    }
}

impl<'a> Deref for Includes<'a> {
    type Target = Content<'a>;

    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

impl<'a> DerefMut for Includes<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.content
    }
}
