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
