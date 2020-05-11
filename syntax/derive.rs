use proc_macro2::Ident;

#[derive(Copy, Clone, PartialEq)]
pub enum Derive {
    Clone,
    Copy,
}

impl Derive {
    pub fn from(ident: &Ident) -> Option<Self> {
        match ident.to_string().as_str() {
            "Clone" => Some(Derive::Clone),
            "Copy" => Some(Derive::Copy),
            _ => None,
        }
    }
}

impl AsRef<str> for Derive {
    fn as_ref(&self) -> &str {
        match self {
            Derive::Clone => "Clone",
            Derive::Copy => "Copy",
        }
    }
}
