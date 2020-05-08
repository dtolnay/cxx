use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::LitStr;

// "folly::File" => `(f, o, l, l, y, (), F, i, l, e)`
pub fn expand(arg: LitStr) -> TokenStream {
    let mut ids = Vec::new();

    for word in arg.value().split("::") {
        if !ids.is_empty() {
            ids.push(quote!(()));
        }
        for ch in word.chars() {
            ids.push(match ch {
                'A'..='Z' | 'a'..='z' => {
                    let t = format_ident!("{}", ch);
                    quote!(::cxx::#t)
                }
                '0'..='9' | '_' => {
                    let t = format_ident!("_{}", ch);
                    quote!(::cxx::#t)
                }
                _ => quote!([(); #ch as _]),
            });
        }
    }

    quote! { (#(#ids,)*) }
}
