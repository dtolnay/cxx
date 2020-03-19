use crate::syntax::atom::Atom::*;
use crate::syntax::{Derive, ExternFn, Ref, Signature, Ty1, Type, Var};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote_spanned, ToTokens};
use syn::Token;

impl ToTokens for Type {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Type::Ident(ident) => {
                if ident == CxxString {
                    let span = ident.span();
                    tokens.extend(quote_spanned!(span=> ::cxx::));
                }
                ident.to_tokens(tokens);
            }
            Type::RustBox(ty) | Type::UniquePtr(ty) => ty.to_tokens(tokens),
            Type::Ref(r) | Type::Str(r) => r.to_tokens(tokens),
            Type::Fn(f) => f.to_tokens(tokens),
            Type::Void(span) => tokens.extend(quote_spanned!(*span=> ())),
        }
    }
}

impl ToTokens for Var {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ident.to_tokens(tokens);
        Token![:](self.ident.span()).to_tokens(tokens);
        self.ty.to_tokens(tokens);
    }
}

impl ToTokens for Ty1 {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if self.name == "UniquePtr" {
            let span = self.name.span();
            tokens.extend(quote_spanned!(span=> ::cxx::));
        }
        self.name.to_tokens(tokens);
        self.langle.to_tokens(tokens);
        self.inner.to_tokens(tokens);
        self.rangle.to_tokens(tokens);
    }
}

impl ToTokens for Ref {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ampersand.to_tokens(tokens);
        self.mutability.to_tokens(tokens);
        self.inner.to_tokens(tokens);
    }
}

impl ToTokens for Derive {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = match self {
            Derive::Clone => "Clone",
            Derive::Copy => "Copy",
        };
        Ident::new(name, Span::call_site()).to_tokens(tokens);
    }
}

impl ToTokens for ExternFn {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.sig.tokens.to_tokens(tokens);
    }
}

impl ToTokens for Signature {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.tokens.to_tokens(tokens);
    }
}
