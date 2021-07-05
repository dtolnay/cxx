use proc_macro2::{Ident, Span};
use std::fmt::{self, Display};

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
    #[cfg(feature = "serde-derive")]
    Deserialize,
    Eq,
    ExternType,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    #[cfg(feature = "serde-derive")]
    Serialize
}

impl Derive {
    pub fn from(ident: &Ident) -> Option<Self> {
        let what = match ident.to_string().as_str() {
            "Clone" => Trait::Clone,
            "Copy" => Trait::Copy,
            "Debug" => Trait::Debug,
            "Default" => Trait::Default,
            #[cfg(feature = "serde-derive")]
            "Deserialize" => Trait::Deserialize,
            "Eq" => Trait::Eq,
            "ExternType" => Trait::ExternType,
            "Hash" => Trait::Hash,
            "Ord" => Trait::Ord,
            "PartialEq" => Trait::PartialEq,
            "PartialOrd" => Trait::PartialOrd,
            #[cfg(feature = "serde-derive")]
            "Serialize" => Trait::Serialize,
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
            #[cfg(feature = "serde-derive")]
            Trait::Deserialize => "Deserialize",
            Trait::Eq => "Eq",
            Trait::ExternType => "ExternType",
            Trait::Hash => "Hash",
            Trait::Ord => "Ord",
            Trait::PartialEq => "PartialEq",
            Trait::PartialOrd => "PartialOrd",
            #[cfg(feature = "serde-derive")]
            Trait::Serialize => "Serialize",
        }
    }
}

impl Display for Derive {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(self.what.as_ref())
    }
}

pub fn contains(derives: &[Derive], query: Trait) -> bool {
    derives.iter().any(|derive| derive.what == query)
}
