use crate::syntax::attrs::find_cxx_bridge_attr;
use crate::syntax::file::Module;
use crate::syntax::namespace::Namespace;
use syn::parse::discouraged::Speculative;
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::{braced, Attribute, Ident, Item, Token, Visibility};

pub(crate) struct File {
    pub modules: Vec<Module>,
}

impl Parse for File {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut modules = Vec::new();
        parse(input, &mut modules)?;
        Ok(File { modules })
    }
}

fn parse(input: ParseStream, modules: &mut Vec<Module>) -> Result<()> {
    input.call(Attribute::parse_inner)?;

    while !input.is_empty() {
        let mut attrs = input.call(Attribute::parse_outer)?;
        let cxx_bridge_attr = find_cxx_bridge_attr(&attrs);

        let ahead = input.fork();
        ahead.parse::<Visibility>()?;
        ahead.parse::<Option<Token![unsafe]>>()?;
        if !ahead.peek(Token![mod]) {
            let item: Item = input.parse()?;
            if cxx_bridge_attr.is_some() {
                return Err(Error::new_spanned(item, "expected a module"));
            }
            continue;
        }

        match cxx_bridge_attr {
            Some(cxx_bridge_attr) => {
                let mut module: Module = input.parse()?;
                module.namespace = Namespace::parse_attr(cxx_bridge_attr)?;
                attrs.extend(module.attrs);
                module.attrs = attrs;
                modules.push(module);
            }
            None => {
                input.advance_to(&ahead);
                input.parse::<Token![mod]>()?;
                input.parse::<Ident>()?;
                let semi: Option<Token![;]> = input.parse()?;
                if semi.is_none() {
                    let content;
                    braced!(content in input);
                    parse(&content, modules)?;
                }
            }
        }
    }

    Ok(())
}
