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
use core::fmt::{Formatter, Result};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{IdentFragment, ToTokens};
use syn::punctuated::Punctuated;
use syn::token::{Brace, Bracket, Paren};
use syn::{Expr, Lifetime, Token, Type as RustType};

pub use self::atom::Atom;
pub use self::derive::Derive;
pub use self::doc::Doc;
pub use self::parse::parse_items;
pub use self::types::Types;

#[derive(Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
/// A C++ identifier in a particular namespace.
/// It is intentional that this does not impl Display,
/// because we want to force users actively to decide whether to output
/// it as a qualified name or as an unqualfiied name.
pub struct QualifiedIdent {
    pub ns: Namespace,
    pub ident: Ident,
}

pub enum Api {
    Include(String),
    Struct(Struct),
    Enum(Enum),
    CxxType(ExternType),
    CxxFunction(ExternFn),
    RustType(ExternType),
    RustFunction(ExternFn),
    TypeAlias(TypeAlias),
    Impl(Impl),
}

pub struct ExternType {
    pub doc: Doc,
    pub type_token: Token![type],
    pub ident: QualifiedIdent,
    pub semi_token: Token![;],
    pub trusted: bool,
}

pub struct Struct {
    pub doc: Doc,
    pub derives: Vec<Derive>,
    pub struct_token: Token![struct],
    pub ident: QualifiedIdent,
    pub brace_token: Brace,
    pub fields: Vec<Var>,
}

pub struct Enum {
    pub doc: Doc,
    pub enum_token: Token![enum],
    pub ident: QualifiedIdent,
    pub brace_token: Brace,
    pub variants: Vec<Variant>,
    pub repr: Atom,
}

pub struct Pair {
    pub cxx: QualifiedIdent,
    pub rust: Ident,
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
    pub ident: QualifiedIdent,
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
    pub ident: Ident, // fields and variables are not namespaced
    pub ty: Type,
}

pub struct Receiver {
    pub ampersand: Token![&],
    pub lifetime: Option<Lifetime>,
    pub mutability: Option<Token![mut]>,
    pub var: Token![self],
    pub ty: QualifiedIdent,
    pub shorthand: bool,
}

pub struct Variant {
    pub ident: Ident,
    pub discriminant: Discriminant,
    pub expr: Option<Expr>,
}

pub enum Type {
    Ident(QualifiedIdent),
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
    pub name: QualifiedIdent,
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

impl Api {
    pub fn get_namespace(&self) -> Option<&Namespace> {
        match self {
            Api::CxxFunction(cfn) => Some(&cfn.ident.cxx.ns),
            Api::CxxType(cty) => Some(&cty.ident.ns),
            Api::Enum(enm) => Some(&enm.ident.ns),
            Api::Struct(strct) => Some(&strct.ident.ns),
            Api::TypeAlias(ta) => Some(&ta.ident.ns),
            Api::RustType(rty) => Some(&rty.ident.ns),
            Api::RustFunction(rfn) => Some(&rfn.ident.cxx.ns),
            Api::Impl(_) | Api::Include(_) => None,
        }
    }
}

impl QualifiedIdent {
    /// Use this constructor if the name is always qualified according to
    /// the namespace.
    pub fn new_never_primitive(ns: &Namespace, ident: Ident) -> Self {
        Self {
            ns: ns.clone(),
            ident,
        }
    }

    /// If there's a chance that the name is not fully-qualified, but
    /// is instead a built-in type (e.g. i32, CxxString, str) then
    /// use this constructor. This is a temporary hack. Eventually we'll
    /// need a later phase to go through and resolve all unresolved
    /// idents according to the current available symbols and 'use'
    /// statements that are in use (which will include an implicit
    /// 'use' statement covering these standard types.) At the moment
    /// there is no such resolution pass, so we aim to try to resolve
    /// all idents at construction time.
    pub fn new_maybe_primitive(ns: &Namespace, ident: Ident) -> Self {
        let is_primitive = Atom::from(&ident).is_some() || ident == "str" || ident == "UniquePtr";
        Self {
            ns: if is_primitive {
                Namespace::none()
            } else {
                ns.clone()
            },
            ident,
        }
    }

    pub fn make_self(span: Span) -> Self {
        QualifiedIdent {
            ns: Namespace::none(),
            ident: Token![Self](span).into(),
        }
    }

    pub fn is_self(&self) -> bool {
        self.ns.is_empty() && self.ident == "Self"
    }

    pub fn span(&self) -> Span {
        self.ident.span()
    }

    fn iter_all_segments(
        &self,
    ) -> std::iter::Chain<std::slice::Iter<Ident>, std::iter::Once<&Ident>> {
        self.ns.iter().chain(std::iter::once(&self.ident))
    }

    fn join(&self, sep: &str) -> String {
        self.iter_all_segments()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join(sep)
    }

    pub fn to_include_guard(&self) -> String {
        self.to_bridge_name()
    }

    pub fn to_bridge_name(&self) -> String {
        self.join("$")
    }

    pub fn to_fully_qualified(&self) -> String {
        format!("::{}", self.join("::"))
    }
}

impl ToTokens for QualifiedIdent {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ident.to_tokens(tokens);
    }
}

impl PartialEq<str> for QualifiedIdent {
    fn eq(&self, other: &str) -> bool {
        self.ns.is_empty() && self.ident == *other
    }
}

impl IdentFragment for QualifiedIdent {
    fn fmt(&self, f: &mut Formatter) -> Result {
        for seg in self.iter_all_segments() {
            f.write_str(&seg.to_string())?;
            f.write_str("__")?;
        }
        Ok(())
    }
}
