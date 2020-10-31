// Functionality that is shared between the cxxbridge macro and the cmd.

pub mod atom;
mod attrs;
pub mod check;
mod derive;
mod discriminant;
mod doc;
pub mod error;
pub mod file;
pub mod ident;
mod impls;
pub mod mangle;
pub mod namespace;
mod parse;
pub mod qualified;
pub mod report;
pub mod set;
pub mod symbol;
mod tokens;
pub mod types;

use self::discriminant::Discriminant;
use self::namespace::Namespace;
use self::parse::kw;
use self::symbol::Symbol;
use proc_macro2::{Ident, Span};
use syn::punctuated::Punctuated;
use syn::token::{Brace, Bracket, Paren};
use syn::{Expr, Lifetime, Token, Type as RustType};

pub use self::atom::Atom;
pub use self::derive::Derive;
pub use self::doc::Doc;
pub use self::parse::parse_items;
pub use self::types::Types;

pub enum Api {
    Include(Include),
    Struct(Struct),
    Enum(Enum),
    CxxType(ExternType),
    CxxFunction(ExternFn),
    RustType(ExternType),
    RustFunction(ExternFn),
    TypeAlias(TypeAlias),
    Impl(Impl),
}

pub struct Include {
    pub path: String,
    pub kind: IncludeKind,
    pub begin_span: Span,
    pub end_span: Span,
}

/// Whether to emit `#include "path"` or `#include <path>`.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum IncludeKind {
    /// `#include "quoted/path/to"`
    Quoted,
    /// `#include <bracketed/path/to>`
    Bracketed,
}

pub struct ExternType {
    pub doc: Doc,
    pub type_token: Token![type],
    pub ident: Pair,
    pub semi_token: Token![;],
    pub trusted: bool,
}

pub struct Struct {
    pub doc: Doc,
    pub derives: Vec<Derive>,
    pub struct_token: Token![struct],
    pub ident: Pair,
    pub brace_token: Brace,
    pub fields: Vec<Var>,
}

pub struct Enum {
    pub doc: Doc,
    pub enum_token: Token![enum],
    pub ident: Pair,
    pub brace_token: Brace,
    pub variants: Vec<Variant>,
    pub repr: Atom,
}

pub struct ExternFn {
    pub lang: Lang,
    pub doc: Doc,
    pub ident: Pair,
    pub sig: Signature,
    pub semi_token: Token![;],
}

pub struct TypeAlias {
    pub doc: Doc,
    pub type_token: Token![type],
    pub ident: Pair,
    pub eq_token: Token![=],
    pub ty: RustType,
    pub semi_token: Token![;],
}

pub struct Impl {
    pub impl_token: Token![impl],
    pub ty: Type,
    pub brace_token: Brace,
}

pub struct Signature {
    pub unsafety: Option<Token![unsafe]>,
    pub fn_token: Token![fn],
    pub receiver: Option<Receiver>,
    pub args: Punctuated<Var, Token![,]>,
    pub ret: Option<Type>,
    pub throws: bool,
    pub paren_token: Paren,
    pub throws_tokens: Option<(kw::Result, Token![<], Token![>])>,
}

#[derive(Eq, PartialEq, Hash)]
pub struct Var {
    pub ident: Ident,
    pub ty: Type,
}

pub struct Receiver {
    pub ampersand: Token![&],
    pub lifetime: Option<Lifetime>,
    pub mutability: Option<Token![mut]>,
    pub var: Token![self],
    pub ty: ResolvableName,
    pub shorthand: bool,
}

pub struct Variant {
    pub ident: Ident,
    pub discriminant: Discriminant,
    pub expr: Option<Expr>,
}

pub enum Type {
    Ident(ResolvableName),
    RustBox(Box<Ty1>),
    RustVec(Box<Ty1>),
    UniquePtr(Box<Ty1>),
    Ref(Box<Ref>),
    Str(Box<Ref>),
    CxxVector(Box<Ty1>),
    Fn(Box<Signature>),
    Void(Span),
    Slice(Box<Slice>),
    SliceRefU8(Box<Ref>),
}

pub struct Ty1 {
    pub name: ResolvableName,
    pub langle: Token![<],
    pub inner: Type,
    pub rangle: Token![>],
}

pub struct Ref {
    pub ampersand: Token![&],
    pub lifetime: Option<Lifetime>,
    pub mutability: Option<Token![mut]>,
    pub inner: Type,
}

pub struct Slice {
    pub bracket: Bracket,
    pub inner: Type,
}

#[derive(Copy, Clone, PartialEq)]
pub enum Lang {
    Cxx,
    Rust,
}

// A type with a defined Rust name and a fully resolved, qualified, namespaced,
// C++ name.
#[derive(Clone)]
pub struct Pair {
    pub cxx: CppName,
    pub rust: Ident,
}

// A C++ identifier in a particular namespace. It is intentional that this does
// not impl Display, because we want to force users actively to decide whether
// to output it as a qualified name or as an unqualfiied name.
#[derive(Clone)]
pub struct CppName {
    pub ns: Namespace,
    pub ident: Ident,
}

// Wrapper for a type which needs to be resolved before it can be printed in
// C++.
#[derive(Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct ResolvableName {
    pub rust: Ident,
}
