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
}

impl Derive {
    pub fn from(ident: &Ident) -> Option<Self> {
        let what = match ident.to_string().as_str() {
            "Clone" => Trait::Clone,
            "Copy" => Trait::Copy,
            "Debug" => Trait::Debug,
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
