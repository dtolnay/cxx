use proc_macro2::Span;
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::{braced, token, Attribute, Ident, Item, Token, Visibility};

pub struct Module {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    // TODO: unsafety
    pub mod_token: Token![mod],
    pub ident: Ident,
    pub brace_token: token::Brace,
    pub content: Vec<Item>,
}

impl Parse for Module {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut attrs = input.call(Attribute::parse_outer)?;
        let vis: Visibility = input.parse()?;
        let mod_token: Token![mod] = input.parse()?;
        let ident: Ident = input.parse()?;

        if input.peek(Token![;]) {
            return Err(Error::new(
                Span::call_site(),
                "#[cxx::bridge] module must have inline contents",
            ))?;
        }

        let content;
        let brace_token = braced!(content in input);
        attrs.extend(content.call(Attribute::parse_inner)?);

        let mut items = Vec::new();
        while !content.is_empty() {
            items.push(content.parse()?);
        }

        Ok(Module {
            attrs,
            vis,
            mod_token,
            ident,
            brace_token,
            content: items,
        })
    }
}
