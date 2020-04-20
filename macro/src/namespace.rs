use crate::syntax::ident;
use crate::syntax::namespace::Namespace;
use quote::IdentFragment;
use std::fmt::{self, Display};
use syn::parse::{Parse, ParseStream, Result};
use syn::{Path, Token};

mod kw {
    syn::custom_keyword!(namespace);
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
        Ok(Namespace::new(segments))
    }
}

impl IdentFragment for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}
