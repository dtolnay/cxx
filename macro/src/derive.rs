use crate::syntax::{Enum, Struct, Trait};
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};

pub fn expand_struct(strct: &Struct) -> TokenStream {
    let mut expanded = TokenStream::new();

    for derive in &strct.derives {
        let span = derive.span;
        match derive.what {
            Trait::Copy => expanded.extend(struct_copy(strct, span)),
            Trait::Clone => expanded.extend(struct_clone(strct, span)),
        }
    }

    expanded
}

pub fn expand_enum(enm: &Enum) -> TokenStream {
    let mut expanded = TokenStream::new();
    let mut has_copy = false;
    let mut has_clone = false;

    for derive in &enm.derives {
        let span = derive.span;
        match derive.what {
            Trait::Copy => {
                expanded.extend(enum_copy(enm, span));
                has_copy = true;
            }
            Trait::Clone => {
                expanded.extend(enum_clone(enm, span));
                has_clone = true;
            }
        }
    }

    let span = enm.name.rust.span();
    if !has_copy {
        expanded.extend(enum_copy(enm, span));
    }
    if !has_clone {
        expanded.extend(enum_clone(enm, span));
    }

    expanded
}

fn struct_copy(strct: &Struct, span: Span) -> TokenStream {
    let ident = &strct.name.rust;

    quote_spanned! {span=>
        impl ::std::marker::Copy for #ident {}
    }
}

fn struct_clone(strct: &Struct, span: Span) -> TokenStream {
    let ident = &strct.name.rust;

    let is_copy = strct
        .derives
        .iter()
        .any(|derive| derive.what == Trait::Copy);

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
        quote_spanned!(span=> #ident {
            #(#fields: ::std::clone::Clone::clone(#values),)*
        })
    };

    quote_spanned! {span=>
        impl ::std::clone::Clone for #ident {
            fn clone(&self) -> Self {
                #body
            }
        }
    }
}

fn enum_copy(enm: &Enum, span: Span) -> TokenStream {
    let ident = &enm.name.rust;

    quote_spanned! {span=>
        impl ::std::marker::Copy for #ident {}
    }
}

fn enum_clone(enm: &Enum, span: Span) -> TokenStream {
    let ident = &enm.name.rust;

    quote_spanned! {span=>
        impl ::std::clone::Clone for #ident {
            fn clone(&self) -> Self {
                *self
            }
        }
    }
}
