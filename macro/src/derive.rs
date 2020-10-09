use crate::syntax::Derive;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

pub struct DeriveAttribute<'a>(pub &'a [Derive]);

impl<'a> ToTokens for DeriveAttribute<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if !self.0.is_empty() {
            let derives = self.0;
            tokens.extend(quote!(#[derive(#(#derives),*)]));
        }
    }
}
