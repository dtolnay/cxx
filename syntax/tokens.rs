use crate::syntax::atom::Atom::*;
use crate::syntax::{
    Array, Atom, Derive, Enum, EnumRepr, ExternFn, ExternType, Impl, Lifetimes, NamedType, Ptr,
    Receiver, Ref, Signature, SliceRef, Struct, Ty1, Type, TypeAlias, Var,
};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote_spanned, ToTokens};
use syn::{token, Token};

impl ToTokens for Type {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Type::Ident(ident) => {
                if ident.rust == Char {
                    let span = ident.rust.span();
                    tokens.extend(quote_spanned!(span=> ::std::os::raw::));
                } else if ident.rust == CxxString {
                    let span = ident.rust.span();
                    tokens.extend(quote_spanned!(span=> ::cxx::));
                }
                ident.to_tokens(tokens);
            }
            Type::RustBox(ty)
            | Type::UniquePtr(ty)
            | Type::SharedPtr(ty)
            | Type::WeakPtr(ty)
            | Type::CxxVector(ty)
            | Type::RustVec(ty) => ty.to_tokens(tokens),
            Type::Ref(r) | Type::Str(r) => r.to_tokens(tokens),
            Type::Ptr(p) => p.to_tokens(tokens),
            Type::Array(a) => a.to_tokens(tokens),
            Type::Fn(f) => f.to_tokens(tokens),
            Type::Void(span) => tokens.extend(quote_spanned!(*span=> ())),
            Type::SliceRef(r) => r.to_tokens(tokens),
        }
    }
}

impl ToTokens for Var {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Var {
            doc: _,
            attrs: _,
            visibility: _,
            name,
            colon_token: _,
            ty,
        } = self;
        name.rust.to_tokens(tokens);
        Token![:](name.rust.span()).to_tokens(tokens);
        ty.to_tokens(tokens);
    }
}

impl ToTokens for Ty1 {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Ty1 {
            name,
            langle,
            inner,
            rangle,
        } = self;
        let span = name.span();
        match name.to_string().as_str() {
            "UniquePtr" | "SharedPtr" | "WeakPtr" | "CxxVector" => {
                tokens.extend(quote_spanned!(span=> ::cxx::));
            }
            "Vec" => {
                tokens.extend(quote_spanned!(span=> ::std::vec::));
            }
            _ => {}
        }
        name.to_tokens(tokens);
        langle.to_tokens(tokens);
        inner.to_tokens(tokens);
        rangle.to_tokens(tokens);
    }
}

impl ToTokens for Ref {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Ref {
            pinned: _,
            option: _,
            ampersand,
            lifetime,
            mutable: _,
            inner,
            pin_tokens,
            option_tokens,
            mutability,
        } = self;
        if let Some((option, langle, _rangle)) = option_tokens {
            tokens.extend(quote_spanned!(option.span=> ::std::option::Option));
            langle.to_tokens(tokens);
        }
        if let Some((pin, langle, _rangle)) = pin_tokens {
            tokens.extend(quote_spanned!(pin.span=> ::std::pin::Pin));
            langle.to_tokens(tokens);
        }
        ampersand.to_tokens(tokens);
        lifetime.to_tokens(tokens);
        mutability.to_tokens(tokens);
        inner.to_tokens(tokens);
        if let Some((_pin, _langle, rangle)) = pin_tokens {
            rangle.to_tokens(tokens);
        }
        if let Some((_option, _langle, rangle)) = option_tokens {
            rangle.to_tokens(tokens);
        }
    }
}

impl ToTokens for Ptr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Ptr {
            star,
            mutable: _,
            inner,
            mutability,
            constness,
        } = self;
        star.to_tokens(tokens);
        mutability.to_tokens(tokens);
        constness.to_tokens(tokens);
        inner.to_tokens(tokens);
    }
}

impl ToTokens for SliceRef {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let SliceRef {
            ampersand,
            lifetime,
            mutable: _,
            bracket,
            inner,
            mutability,
        } = self;
        ampersand.to_tokens(tokens);
        lifetime.to_tokens(tokens);
        mutability.to_tokens(tokens);
        bracket.surround(tokens, |tokens| {
            inner.to_tokens(tokens);
        });
    }
}

impl ToTokens for Array {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Array {
            bracket,
            inner,
            semi_token,
            len: _,
            len_token,
        } = self;
        bracket.surround(tokens, |tokens| {
            inner.to_tokens(tokens);
            semi_token.to_tokens(tokens);
            len_token.to_tokens(tokens);
        });
    }
}

impl ToTokens for Atom {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        Ident::new(self.as_ref(), Span::call_site()).to_tokens(tokens);
    }
}

impl ToTokens for Derive {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        Ident::new(self.what.as_ref(), self.span).to_tokens(tokens);
    }
}

impl ToTokens for ExternType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        // Notional token range for error reporting purposes.
        self.type_token.to_tokens(tokens);
        self.name.rust.to_tokens(tokens);
        self.generics.to_tokens(tokens);
    }
}

impl ToTokens for TypeAlias {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        // Notional token range for error reporting purposes.
        self.type_token.to_tokens(tokens);
        self.name.rust.to_tokens(tokens);
        self.generics.to_tokens(tokens);
    }
}

impl ToTokens for Struct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        // Notional token range for error reporting purposes.
        self.struct_token.to_tokens(tokens);
        self.name.rust.to_tokens(tokens);
        self.generics.to_tokens(tokens);
    }
}

impl ToTokens for Enum {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        // Notional token range for error reporting purposes.
        self.enum_token.to_tokens(tokens);
        self.name.rust.to_tokens(tokens);
        self.generics.to_tokens(tokens);
    }
}

impl ToTokens for ExternFn {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        // Notional token range for error reporting purposes.
        self.unsafety.to_tokens(tokens);
        self.sig.fn_token.to_tokens(tokens);
        self.semi_token.to_tokens(tokens);
    }
}

impl ToTokens for Impl {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Impl {
            impl_token,
            impl_generics,
            negative: _,
            ty,
            ty_generics: _,
            brace_token,
            negative_token,
        } = self;
        impl_token.to_tokens(tokens);
        impl_generics.to_tokens(tokens);
        negative_token.to_tokens(tokens);
        ty.to_tokens(tokens);
        brace_token.surround(tokens, |_tokens| {});
    }
}

impl ToTokens for Lifetimes {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Lifetimes {
            lt_token,
            lifetimes,
            gt_token,
        } = self;
        lt_token.to_tokens(tokens);
        lifetimes.to_tokens(tokens);
        gt_token.to_tokens(tokens);
    }
}

impl ToTokens for Signature {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Signature {
            unsafety: _,
            fn_token,
            generics: _,
            receiver: _,
            args,
            ret,
            throws: _,
            paren_token,
            throws_tokens,
        } = self;
        fn_token.to_tokens(tokens);
        paren_token.surround(tokens, |tokens| {
            args.to_tokens(tokens);
        });
        if let Some(ret) = ret {
            Token![->](paren_token.span).to_tokens(tokens);
            if let Some((result, langle, rangle)) = throws_tokens {
                result.to_tokens(tokens);
                langle.to_tokens(tokens);
                ret.to_tokens(tokens);
                rangle.to_tokens(tokens);
            } else {
                ret.to_tokens(tokens);
            }
        } else if let Some((result, langle, rangle)) = throws_tokens {
            Token![->](paren_token.span).to_tokens(tokens);
            result.to_tokens(tokens);
            langle.to_tokens(tokens);
            token::Paren(langle.span).surround(tokens, |_| ());
            rangle.to_tokens(tokens);
        }
    }
}

impl ToTokens for EnumRepr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            EnumRepr::Native { atom, repr_type: _ } => atom.to_tokens(tokens),
            EnumRepr::Foreign { rust_type } => rust_type.to_tokens(tokens),
        }
    }
}

impl ToTokens for NamedType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let NamedType { rust, generics } = self;
        rust.to_tokens(tokens);
        generics.to_tokens(tokens);
    }
}

pub struct ReceiverType<'a>(&'a Receiver);
pub struct ReceiverTypeSelf<'a>(&'a Receiver);

impl Receiver {
    // &TheType
    pub fn ty(&self) -> ReceiverType {
        ReceiverType(self)
    }

    // &Self
    pub fn ty_self(&self) -> ReceiverTypeSelf {
        ReceiverTypeSelf(self)
    }
}

impl ToTokens for ReceiverType<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Receiver {
            pinned: _,
            ampersand,
            lifetime,
            mutable: _,
            var: _,
            colon_token: _,
            ty,
            shorthand: _,
            pin_tokens,
            mutability,
        } = &self.0;
        if let Some((pin, langle, _rangle)) = pin_tokens {
            tokens.extend(quote_spanned!(pin.span=> ::std::pin::Pin));
            langle.to_tokens(tokens);
        }
        ampersand.to_tokens(tokens);
        lifetime.to_tokens(tokens);
        mutability.to_tokens(tokens);
        ty.to_tokens(tokens);
        if let Some((_pin, _langle, rangle)) = pin_tokens {
            rangle.to_tokens(tokens);
        }
    }
}

impl ToTokens for ReceiverTypeSelf<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Receiver {
            pinned: _,
            ampersand,
            lifetime,
            mutable: _,
            var: _,
            colon_token: _,
            ty,
            shorthand: _,
            pin_tokens,
            mutability,
        } = &self.0;
        if let Some((pin, langle, _rangle)) = pin_tokens {
            tokens.extend(quote_spanned!(pin.span=> ::std::pin::Pin));
            langle.to_tokens(tokens);
        }
        ampersand.to_tokens(tokens);
        lifetime.to_tokens(tokens);
        mutability.to_tokens(tokens);
        Token![Self](ty.rust.span()).to_tokens(tokens);
        if let Some((_pin, _langle, rangle)) = pin_tokens {
            rangle.to_tokens(tokens);
        }
    }
}
