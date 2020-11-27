use crate::syntax::{Struct, Trait};
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};

pub fn expand_struct(strct: &Struct) -> TokenStream {
    let ident = &strct.name.rust;
    let mut expanded = TokenStream::new();

    let is_copy = strct
        .derives
        .iter()
        .any(|derive| derive.what == Trait::Copy);

    for derive in &strct.derives {
        match derive.what {
            Trait::Copy => {
                expanded.extend(quote_spanned! {derive.span=>
                    impl ::std::marker::Copy for #ident {}
                });
            }
            Trait::Clone => {
                let body = if is_copy {
                    quote!(*self)
                } else {
                    let fields = strct.fields.iter().map(|field| &field.ident);
                    let values = strct.fields.iter().map(|field| {
                        let ident = &field.ident;
                        let ty = field.ty.to_token_stream();
                        let span = ty.into_iter().last().unwrap().span();
                        quote_spanned!(span=> &self.#ident)
                    });
                    quote_spanned!(derive.span=> #ident {
                        #(#fields: ::std::clone::Clone::clone(#values),)*
                    })
                };
                expanded.extend(quote_spanned! {derive.span=>
                    impl ::std::clone::Clone for #ident {
                        fn clone(&self) -> Self {
                            #body
                        }
                    }
                });
            }
        }
    }

    expanded
}
