use syn::parse::{Parse, ParseStream, Result};
use syn::{Attribute, Item};

pub struct File {
    pub attrs: Vec<Attribute>,
    pub items: Vec<Item>,
}

impl Parse for File {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_inner)?;

        let mut items = Vec::new();
        while !input.is_empty() {
            items.push(input.parse()?);
        }

        Ok(File { attrs, items })
    }
}
