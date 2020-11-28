use proc_macro2::{Ident, Span};

#[derive(Copy, Clone)]
pub struct Derive {
    pub what: Trait,
    pub span: Span,
}

#[derive(Copy, Clone, PartialEq)]
pub enum Trait {
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
}

impl Derive {
    pub fn from(ident: &Ident) -> Option<Self> {
        let what = match ident.to_string().as_str() {
            "Clone" => Trait::Clone,
            "Copy" => Trait::Copy,
            "Debug" => Trait::Debug,
            "Default" => Trait::Default,
            "Eq" => Trait::Eq,
            "Hash" => Trait::Hash,
            "Ord" => Trait::Ord,
            "PartialEq" => Trait::PartialEq,
            "PartialOrd" => Trait::PartialOrd,
            _ => return None,
        };
        let span = ident.span();
        Some(Derive { what, span })
    }
}

impl PartialEq<Trait> for Derive {
    fn eq(&self, other: &Trait) -> bool {
        self.what == *other
    }
}

impl AsRef<str> for Trait {
    fn as_ref(&self) -> &str {
        match self {
            Trait::Clone => "Clone",
            Trait::Copy => "Copy",
            Trait::Debug => "Debug",
            Trait::Default => "Default",
            Trait::Eq => "Eq",
            Trait::Hash => "Hash",
            Trait::Ord => "Ord",
            Trait::PartialEq => "PartialEq",
            Trait::PartialOrd => "PartialOrd",
        }
    }
}

pub fn contains(derives: &[Derive], query: Trait) -> bool {
    derives.iter().any(|derive| derive.what == query)
}
