use crate::syntax::atom::Atom::*;
use crate::syntax::{Derive, ExternFn, Receiver, Ref, Signature, Slice, Ty1, Type, Var};
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
            Type::RustBox(ty) | Type::UniquePtr(ty) | Type::CxxVector(ty) | Type::RustVec(ty) => {
                ty.to_tokens(tokens)
            }
            Type::Ref(r) | Type::Str(r) | Type::SliceRefU8(r) => r.to_tokens(tokens),
            Type::Slice(s) => s.to_tokens(tokens),
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
        // Do not add cxx namespace to Vector since we're defining it in the user crate
        if self.name == "UniquePtr" || self.name == "RustVec" {
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
        self.lifetime.to_tokens(tokens);
        self.mutability.to_tokens(tokens);
        self.inner.to_tokens(tokens);
    }
}

impl ToTokens for Slice {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.bracket.surround(tokens, |tokens| {
            self.inner.to_tokens(tokens);
        });
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
        // Notional token range for error reporting purposes.
        self.sig.fn_token.to_tokens(tokens);
        self.semi_token.to_tokens(tokens);
    }
}

impl ToTokens for Signature {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.fn_token.to_tokens(tokens);
        self.paren_token.surround(tokens, |tokens| {
            self.args.to_tokens(tokens);
        });
        if let Some(ret) = &self.ret {
            Token![->](self.paren_token.span).to_tokens(tokens);
            if let Some((result, langle, rangle)) = self.throws_tokens {
                result.to_tokens(tokens);
                langle.to_tokens(tokens);
                ret.to_tokens(tokens);
                rangle.to_tokens(tokens);
            } else {
                ret.to_tokens(tokens);
            }
        }
    }
}

pub struct ReceiverType<'a>(&'a Receiver);

impl Receiver {
    // &TheType
    pub fn ty(&self) -> ReceiverType {
        ReceiverType(self)
    }
}

impl ToTokens for ReceiverType<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.ampersand.to_tokens(tokens);
        self.0.lifetime.to_tokens(tokens);
        self.0.mutability.to_tokens(tokens);
        self.0.ty.to_tokens(tokens);
    }
}
