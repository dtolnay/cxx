use crate::syntax::ident;
use quote::IdentFragment;
use std::fmt::{self, Display};
use std::slice::Iter;
use syn::parse::{Parse, ParseStream, Result};
use syn::{Path, Token};

mod kw {
    syn::custom_keyword!(namespace);
}

#[derive(Clone)]
pub struct Namespace {
    segments: Vec<String>,
}

impl Namespace {
    pub fn none() -> Self {
        Namespace {
            segments: Vec::new(),
        }
    }

    pub fn iter(&self) -> Iter<String> {
        self.segments.iter()
    }
}

impl Parse for Namespace {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut segments = Vec::new();
        if !input.is_empty() {
            input.parse::<kw::namespace>()?;
            input.parse::<Token![=]>()?;
            let path = input.call(Path::parse_mod_style)?;
            for segment in path.segments {
                ident::check(&segment.ident)?;
                segments.push(segment.ident.to_string());
            }
            input.parse::<Option<Token![,]>>()?;
        }
        Ok(Namespace { segments })
    }
}

impl Display for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for segment in self {
            f.write_str(segment)?;
            f.write_str("$")?;
        }
        Ok(())
    }
}

impl IdentFragment for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl<'a> IntoIterator for &'a Namespace {
    type Item = &'a String;
    type IntoIter = Iter<'a, String>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
