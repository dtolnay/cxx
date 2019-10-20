use crate::syntax::ident;
use std::fmt::{self, Display};
use syn::parse::{Parse, ParseStream, Result};
use syn::{Path, Token};

mod kw {
    syn::custom_keyword!(namespace);
}

pub struct Namespace {
    segments: Vec<String>,
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
        for segment in &self.segments {
            f.write_str(segment)?;
            f.write_str("$")?;
        }
        Ok(())
    }
}
