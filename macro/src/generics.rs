use crate::syntax::types::ConditionalImpl;
use crate::syntax::{Lifetimes, Type, Types};
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::Lifetime;

/// Gets `(impl_generics, ty_generics)` pair that can be used when generating an
/// `impl` for a generic type:
///
/// ```ignore
/// quote! { impl #impl_generics SomeTrait for #inner #ty_generics }
/// ```
pub(crate) fn split_for_impl<'a>(
    inner: &'a Type,
    conditional_impl: &ConditionalImpl<'a>,
    types: &'a Types,
) -> (&'a Lifetimes, Option<&'a Lifetimes>) {
    match conditional_impl.explicit_impl {
        Some(explicit_impl) => {
            let impl_generics = &explicit_impl.impl_generics;
            let ty_generics = None; // already covered via `#inner`
            (impl_generics, ty_generics)
        }
        None => {
            // Check whether explicit generics are present. In the example
            // below, there are not explicit generics in the return type.
            //
            //     mod ffi {
            //         unsafe extern "C++" {
            //             type Borrowed<'a>;
            //             fn borrowed(arg: &i32) -> UniquePtr<Borrowed>;
            //         }
            //     }
            //
            // But this could have also been spelled with explicit generics:
            //
            //             fn borrowed<'a>(arg: &'a i32) -> UniquePtr<Borrowed<'a>>;
            let explicit_generics = get_generic_lifetimes(inner);
            if explicit_generics.lifetimes.is_empty() {
                // In the example above, we want to use generics from `type Borrowed<'a>`.
                let resolved_generics = resolve_generic_lifetimes(inner, types);
                (resolved_generics, Some(resolved_generics))
            } else {
                let ty_generics = None; // already covered via `#inner`
                (explicit_generics, ty_generics)
            }
        }
    }
}

/// Gets explicit (not elided) lifetimes from `ty`. This will recurse into type
/// arguments as in `CxxVector<Borrowed<'a>>`.
fn get_generic_lifetimes(ty: &Type) -> &Lifetimes {
    match ty {
        Type::Ident(named_type) => &named_type.generics,
        Type::CxxVector(ty1) => get_generic_lifetimes(&ty1.inner),
        _ => unreachable!("syntax/check.rs should reject other types"),
    }
}

/// Gets lifetimes from the declaration of `ty`'s local type. For example, if
/// `ty` represents `CxxVector<Borrowed>` in the following module, this will
/// return the `<'a>`.
///
/// ```rust,ignore
/// unsafe extern "C++" {
///     type Borrowed<'a>;
///     fn borrowed(arg: &i32) -> CxxVector<Borrowed>;
/// }
/// ```
fn resolve_generic_lifetimes<'a>(ty: &Type, types: &Types<'a>) -> &'a Lifetimes {
    match ty {
        Type::Ident(named_type) => types.resolve(&named_type.rust).generics,
        Type::CxxVector(ty1) => resolve_generic_lifetimes(&ty1.inner, types),
        _ => unreachable!("syntax/check.rs should reject other types"),
    }
}

pub(crate) fn concise_rust_name(ty: &Type) -> String {
    match ty {
        Type::Ident(named_type) => named_type.rust.to_string(),
        _ => unreachable!("syntax/check.rs should reject other types"),
    }
}

pub(crate) struct UnderscoreLifetimes<'a> {
    generics: &'a Lifetimes,
}

impl Lifetimes {
    pub(crate) fn to_underscore_lifetimes(&self) -> UnderscoreLifetimes {
        UnderscoreLifetimes { generics: self }
    }
}

impl<'a> ToTokens for UnderscoreLifetimes<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Lifetimes {
            lt_token,
            lifetimes,
            gt_token,
        } = self.generics;
        lt_token.to_tokens(tokens);
        for pair in lifetimes.pairs() {
            let (lifetime, punct) = pair.into_tuple();
            let lifetime = Lifetime::new("'_", lifetime.span());
            lifetime.to_tokens(tokens);
            punct.to_tokens(tokens);
        }
        gt_token.to_tokens(tokens);
    }
}
