// Functionality that is shared between the cxxbridge macro and the cmd.

pub mod atom;
mod attrs;
pub mod check;
mod derive;
mod discriminant;
mod doc;
pub mod error;
pub mod ident;
mod impls;
pub mod mangle;
pub mod namespace;
mod parse;
pub mod report;
pub mod set;
pub mod symbol;
mod tokens;
pub mod types;

use self::discriminant::Discriminant;
use self::parse::kw;
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
    Include(String),
    Struct(Struct),
    Enum(Enum),
    CxxType(ExternType),
    CxxFunction(ExternFn),
    RustType(ExternType),
    RustFunction(ExternFn),
    TypeAlias(TypeAlias),
}

pub struct ExternType {
    pub doc: Doc,
    pub type_token: Token![type],
    pub ident: Ident,
    pub semi_token: Token![;],
}

pub struct Struct {
    pub doc: Doc,
    pub derives: Vec<Derive>,
    pub struct_token: Token![struct],
    pub ident: Ident,
    pub brace_token: Brace,
    pub fields: Vec<Var>,
}

pub struct Enum {
    pub doc: Doc,
    pub enum_token: Token![enum],
    pub ident: Ident,
    pub brace_token: Brace,
    pub variants: Vec<Variant>,
    pub repr: Atom,
}

pub struct ExternFn {
    pub lang: Lang,
    pub doc: Doc,
    pub ident: Ident,
    pub sig: Signature,
    pub semi_token: Token![;],
}

pub struct TypeAlias {
    pub type_token: Token![type],
    pub ident: Ident,
    pub eq_token: Token![=],
    pub ty: RustType,
    pub semi_token: Token![;],
}

pub struct Signature {
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
    pub ty: Ident,
    pub shorthand: bool,
}

pub struct Variant {
    pub ident: Ident,
    pub discriminant: Discriminant,
    pub expr: Option<Expr>,
}

pub enum Type {
    Ident(Ident),
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
    pub name: Ident,
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
