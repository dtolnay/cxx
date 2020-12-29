use crate::syntax::discriminant::DiscriminantSet;
use crate::syntax::file::{Item, ItemForeignMod};
use crate::syntax::report::Errors;
use crate::syntax::Atom::*;
use crate::syntax::{
    attrs, error, Api, Array, Derive, Doc, Enum, ExternFn, ExternType, Impl, Include, IncludeKind,
    Lang, Lifetimes, Namespace, Pair, Receiver, Ref, RustName, Signature, SliceRef, Struct, Ty1,
    Type, TypeAlias, Var, Variant,
};
use proc_macro2::{Delimiter, Group, Span, TokenStream, TokenTree};
use quote::{format_ident, quote, quote_spanned};
use syn::parse::{ParseStream, Parser};
use syn::punctuated::Punctuated;
use syn::{
    Abi, Attribute, Error, Expr, Fields, FnArg, ForeignItem, ForeignItemFn, ForeignItemType,
    GenericArgument, GenericParam, Generics, Ident, ItemEnum, ItemImpl, ItemStruct, Lit, LitStr,
    Pat, PathArguments, Result, ReturnType, Token, TraitBound, TraitBoundModifier,
    Type as RustType, TypeArray, TypeBareFn, TypeParamBound, TypePath, TypeReference,
    Variant as RustVariant,
};

pub mod kw {
    syn::custom_keyword!(Pin);
    syn::custom_keyword!(Result);
}

pub fn parse_items(
    cx: &mut Errors,
    items: Vec<Item>,
    trusted: bool,
    namespace: &Namespace,
) -> Vec<Api> {
    let mut apis = Vec::new();
    for item in items {
        match item {
            Item::Struct(item) => match parse_struct(cx, item, namespace) {
                Ok(strct) => apis.push(strct),
                Err(err) => cx.push(err),
            },
            Item::Enum(item) => match parse_enum(cx, item, namespace) {
                Ok(enm) => apis.push(enm),
                Err(err) => cx.push(err),
            },
            Item::ForeignMod(foreign_mod) => {
                parse_foreign_mod(cx, foreign_mod, &mut apis, trusted, namespace)
            }
            Item::Impl(item) => match parse_impl(item) {
                Ok(imp) => apis.push(imp),
                Err(err) => cx.push(err),
            },
            Item::Use(item) => cx.error(item, error::USE_NOT_ALLOWED),
            Item::Other(item) => cx.error(item, "unsupported item"),
        }
    }
    apis
}

fn parse_struct(cx: &mut Errors, item: ItemStruct, namespace: &Namespace) -> Result<Api> {
    let mut doc = Doc::new();
    let mut derives = Vec::new();
    let mut namespace = namespace.clone();
    let mut cxx_name = None;
    let mut rust_name = None;
    attrs::parse(
        cx,
        &item.attrs,
        attrs::Parser {
            doc: Some(&mut doc),
            derives: Some(&mut derives),
            namespace: Some(&mut namespace),
            cxx_name: Some(&mut cxx_name),
            rust_name: Some(&mut rust_name),
            ..Default::default()
        },
    );

    let generics = &item.generics;
    if !generics.params.is_empty() || generics.where_clause.is_some() {
        let struct_token = item.struct_token;
        let ident = &item.ident;
        let where_clause = &generics.where_clause;
        let span = quote!(#struct_token #ident #generics #where_clause);
        return Err(Error::new_spanned(
            span,
            "struct with generic parameters is not supported yet",
        ));
    }

    let named_fields = match item.fields {
        Fields::Named(fields) => fields,
        Fields::Unit => return Err(Error::new_spanned(item, "unit structs are not supported")),
        Fields::Unnamed(_) => {
            return Err(Error::new_spanned(item, "tuple structs are not supported"));
        }
    };

    let mut fields = Vec::new();
    for field in named_fields.named {
        let ident = field.ident.unwrap();
        let ty = match parse_type(&field.ty) {
            Ok(ty) => ty,
            Err(err) => {
                cx.push(err);
                continue;
            }
        };
        fields.push(Var { ident, ty });
    }

    let struct_token = item.struct_token;
    let name = pair(namespace, &item.ident, cxx_name, rust_name);
    let brace_token = named_fields.brace_token;

    Ok(Api::Struct(Struct {
        doc,
        derives,
        struct_token,
        name,
        brace_token,
        fields,
    }))
}

fn parse_enum(cx: &mut Errors, item: ItemEnum, namespace: &Namespace) -> Result<Api> {
    let mut doc = Doc::new();
    let mut derives = Vec::new();
    let mut repr = None;
    let mut namespace = namespace.clone();
    let mut cxx_name = None;
    let mut rust_name = None;
    attrs::parse(
        cx,
        &item.attrs,
        attrs::Parser {
            doc: Some(&mut doc),
            derives: Some(&mut derives),
            repr: Some(&mut repr),
            namespace: Some(&mut namespace),
            cxx_name: Some(&mut cxx_name),
            rust_name: Some(&mut rust_name),
            ..Default::default()
        },
    );

    let generics = &item.generics;
    if !generics.params.is_empty() || generics.where_clause.is_some() {
        let enum_token = item.enum_token;
        let ident = &item.ident;
        let where_clause = &generics.where_clause;
        let span = quote!(#enum_token #ident #generics #where_clause);
        return Err(Error::new_spanned(
            span,
            "enums with generic parameters are not allowed",
        ));
    }

    let mut variants = Vec::new();
    let mut discriminants = DiscriminantSet::new(repr);
    for variant in item.variants {
        match parse_variant(cx, variant, &mut discriminants) {
            Ok(variant) => variants.push(variant),
            Err(err) => cx.push(err),
        }
    }

    let enum_token = item.enum_token;
    let brace_token = item.brace_token;

    let explicit_repr = repr.is_some();
    let mut repr = U8;
    match discriminants.inferred_repr() {
        Ok(inferred) => repr = inferred,
        Err(err) => {
            let span = quote_spanned!(brace_token.span=> #enum_token {});
            cx.error(span, err);
            variants.clear();
        }
    }

    let name = pair(namespace, &item.ident, cxx_name, rust_name);
    let repr_ident = Ident::new(repr.as_ref(), Span::call_site());
    let repr_type = Type::Ident(RustName::new(repr_ident));

    Ok(Api::Enum(Enum {
        doc,
        derives,
        enum_token,
        name,
        brace_token,
        variants,
        repr,
        repr_type,
        explicit_repr,
    }))
}

fn parse_variant(
    cx: &mut Errors,
    variant: RustVariant,
    discriminants: &mut DiscriminantSet,
) -> Result<Variant> {
    let mut cxx_name = None;
    let mut rust_name = None;
    attrs::parse(
        cx,
        &variant.attrs,
        attrs::Parser {
            cxx_name: Some(&mut cxx_name),
            rust_name: Some(&mut rust_name),
            ..Default::default()
        },
    );

    match variant.fields {
        Fields::Unit => {}
        _ => {
            let msg = "enums with data are not supported yet";
            return Err(Error::new_spanned(variant, msg));
        }
    }

    let expr = variant.discriminant.as_ref().map(|(_, expr)| expr);
    let try_discriminant = match &expr {
        Some(lit) => discriminants.insert(lit),
        None => discriminants.insert_next(),
    };
    let discriminant = match try_discriminant {
        Ok(discriminant) => discriminant,
        Err(err) => return Err(Error::new_spanned(variant, err)),
    };

    let name = pair(Namespace::ROOT, &variant.ident, cxx_name, rust_name);
    let expr = variant.discriminant.map(|(_, expr)| expr);

    Ok(Variant {
        name,
        discriminant,
        expr,
    })
}

fn parse_foreign_mod(
    cx: &mut Errors,
    foreign_mod: ItemForeignMod,
    out: &mut Vec<Api>,
    trusted: bool,
    namespace: &Namespace,
) {
    let lang = match parse_lang(&foreign_mod.abi) {
        Ok(lang) => lang,
        Err(err) => return cx.push(err),
    };

    match lang {
        Lang::Rust => {
            if foreign_mod.unsafety.is_some() {
                let unsafety = foreign_mod.unsafety;
                let abi = &foreign_mod.abi;
                let span = quote!(#unsafety #abi);
                cx.error(span, "extern \"Rust\" block does not need to be unsafe");
            }
        }
        Lang::Cxx => {}
    }

    let trusted = trusted || foreign_mod.unsafety.is_some();

    let mut namespace = namespace.clone();
    attrs::parse(
        cx,
        &foreign_mod.attrs,
        attrs::Parser {
            namespace: Some(&mut namespace),
            ..Default::default()
        },
    );

    let mut items = Vec::new();
    for foreign in &foreign_mod.items {
        match foreign {
            ForeignItem::Type(foreign) => {
                let ety = parse_extern_type(cx, foreign, lang, trusted, &namespace);
                items.push(ety);
            }
            ForeignItem::Fn(foreign) => {
                match parse_extern_fn(cx, foreign, lang, trusted, &namespace) {
                    Ok(efn) => items.push(efn),
                    Err(err) => cx.push(err),
                }
            }
            ForeignItem::Macro(foreign) if foreign.mac.path.is_ident("include") => {
                match foreign.mac.parse_body_with(parse_include) {
                    Ok(include) => items.push(Api::Include(include)),
                    Err(err) => cx.push(err),
                }
            }
            ForeignItem::Verbatim(tokens) => {
                match parse_extern_verbatim(cx, tokens, lang, trusted, &namespace) {
                    Ok(api) => items.push(api),
                    Err(err) => cx.push(err),
                }
            }
            _ => cx.error(foreign, "unsupported foreign item"),
        }
    }

    if !trusted
        && items.iter().any(|api| match api {
            Api::CxxFunction(efn) => efn.unsafety.is_none(),
            _ => false,
        })
    {
        cx.error(
            foreign_mod.abi,
            "block must be declared `unsafe extern \"C++\"` if it contains any safe-to-call C++ functions",
        );
    }

    let mut types = items.iter().filter_map(|item| match item {
        Api::CxxType(ety) | Api::RustType(ety) => Some(&ety.name),
        Api::TypeAlias(alias) => Some(&alias.name),
        _ => None,
    });
    if let (Some(single_type), None) = (types.next(), types.next()) {
        let single_type = single_type.clone();
        for item in &mut items {
            if let Api::CxxFunction(efn) | Api::RustFunction(efn) = item {
                if let Some(receiver) = &mut efn.receiver {
                    if receiver.ty.rust == "Self" {
                        receiver.ty.rust = single_type.rust.clone();
                    }
                }
            }
        }
    }

    out.extend(items);
}

fn parse_lang(abi: &Abi) -> Result<Lang> {
    let name = match &abi.name {
        Some(name) => name,
        None => {
            return Err(Error::new_spanned(
                abi,
                "ABI name is required, extern \"C++\" or extern \"Rust\"",
            ));
        }
    };

    match name.value().as_str() {
        "C++" => Ok(Lang::Cxx),
        "Rust" => Ok(Lang::Rust),
        _ => Err(Error::new_spanned(
            abi,
            "unrecognized ABI, requires either \"C++\" or \"Rust\"",
        )),
    }
}

fn parse_extern_type(
    cx: &mut Errors,
    foreign_type: &ForeignItemType,
    lang: Lang,
    trusted: bool,
    namespace: &Namespace,
) -> Api {
    let mut doc = Doc::new();
    let mut derives = Vec::new();
    let mut namespace = namespace.clone();
    let mut cxx_name = None;
    let mut rust_name = None;
    attrs::parse(
        cx,
        &foreign_type.attrs,
        attrs::Parser {
            doc: Some(&mut doc),
            derives: Some(&mut derives),
            namespace: Some(&mut namespace),
            cxx_name: Some(&mut cxx_name),
            rust_name: Some(&mut rust_name),
            ..Default::default()
        },
    );

    let type_token = foreign_type.type_token;
    let name = pair(namespace, &foreign_type.ident, cxx_name, rust_name);
    let generics = Lifetimes {
        lt_token: None,
        lifetimes: Punctuated::new(),
        gt_token: None,
    };
    let colon_token = None;
    let bounds = Vec::new();
    let semi_token = foreign_type.semi_token;

    (match lang {
        Lang::Cxx => Api::CxxType,
        Lang::Rust => Api::RustType,
    })(ExternType {
        lang,
        doc,
        derives,
        type_token,
        name,
        generics,
        colon_token,
        bounds,
        semi_token,
        trusted,
    })
}

fn parse_extern_fn(
    cx: &mut Errors,
    foreign_fn: &ForeignItemFn,
    lang: Lang,
    trusted: bool,
    namespace: &Namespace,
) -> Result<Api> {
    let mut doc = Doc::new();
    let mut namespace = namespace.clone();
    let mut cxx_name = None;
    let mut rust_name = None;
    attrs::parse(
        cx,
        &foreign_fn.attrs,
        attrs::Parser {
            doc: Some(&mut doc),
            namespace: Some(&mut namespace),
            cxx_name: Some(&mut cxx_name),
            rust_name: Some(&mut rust_name),
            ..Default::default()
        },
    );

    let generics = &foreign_fn.sig.generics;
    if generics.where_clause.is_some()
        || generics.params.iter().any(|param| match param {
            GenericParam::Lifetime(lifetime) => !lifetime.bounds.is_empty(),
            GenericParam::Type(_) | GenericParam::Const(_) => true,
        })
    {
        return Err(Error::new_spanned(
            foreign_fn,
            "extern function with generic parameters is not supported yet",
        ));
    }

    if let Some(variadic) = &foreign_fn.sig.variadic {
        return Err(Error::new_spanned(
            variadic,
            "variadic function is not supported yet",
        ));
    }

    if foreign_fn.sig.asyncness.is_some() {
        return Err(Error::new_spanned(
            foreign_fn,
            "async function is not directly supported yet, but see https://cxx.rs/async.html for a working approach",
        ));
    }

    if foreign_fn.sig.constness.is_some() {
        return Err(Error::new_spanned(
            foreign_fn,
            "const extern function is not supported",
        ));
    }

    if let Some(abi) = &foreign_fn.sig.abi {
        return Err(Error::new_spanned(
            abi,
            "explicit ABI on extern function is not supported",
        ));
    }

    let mut receiver = None;
    let mut args = Punctuated::new();
    for arg in foreign_fn.sig.inputs.pairs() {
        let (arg, comma) = arg.into_tuple();
        match arg {
            FnArg::Receiver(arg) => {
                if let Some((ampersand, lifetime)) = &arg.reference {
                    receiver = Some(Receiver {
                        pinned: false,
                        ampersand: *ampersand,
                        lifetime: lifetime.clone(),
                        mutable: arg.mutability.is_some(),
                        var: arg.self_token,
                        ty: RustName::new(Ident::new("Self", arg.self_token.span)),
                        shorthand: true,
                        pin_tokens: None,
                        mutability: arg.mutability,
                    });
                    continue;
                }
                return Err(Error::new_spanned(arg, "unsupported signature"));
            }
            FnArg::Typed(arg) => {
                let ident = match arg.pat.as_ref() {
                    Pat::Ident(pat) => pat.ident.clone(),
                    Pat::Wild(pat) => {
                        Ident::new(&format!("arg{}", args.len()), pat.underscore_token.span)
                    }
                    _ => return Err(Error::new_spanned(arg, "unsupported signature")),
                };
                let ty = parse_type(&arg.ty)?;
                if ident != "self" {
                    args.push_value(Var { ident, ty });
                    if let Some(comma) = comma {
                        args.push_punct(*comma);
                    }
                    continue;
                }
                if let Type::Ref(reference) = ty {
                    if let Type::Ident(ident) = reference.inner {
                        receiver = Some(Receiver {
                            pinned: reference.pinned,
                            ampersand: reference.ampersand,
                            lifetime: reference.lifetime,
                            mutable: reference.mutable,
                            var: Token![self](ident.rust.span()),
                            ty: ident,
                            shorthand: false,
                            pin_tokens: reference.pin_tokens,
                            mutability: reference.mutability,
                        });
                        continue;
                    }
                }
                return Err(Error::new_spanned(arg, "unsupported method receiver"));
            }
        }
    }

    let mut throws_tokens = None;
    let ret = parse_return_type(&foreign_fn.sig.output, &mut throws_tokens)?;
    let throws = throws_tokens.is_some();
    let unsafety = foreign_fn.sig.unsafety;
    let fn_token = foreign_fn.sig.fn_token;
    let name = pair(namespace, &foreign_fn.sig.ident, cxx_name, rust_name);
    let generics = generics.clone();
    let paren_token = foreign_fn.sig.paren_token;
    let semi_token = foreign_fn.semi_token;

    Ok(match lang {
        Lang::Cxx => Api::CxxFunction,
        Lang::Rust => Api::RustFunction,
    }(ExternFn {
        lang,
        doc,
        name,
        sig: Signature {
            unsafety,
            fn_token,
            generics,
            receiver,
            args,
            ret,
            throws,
            paren_token,
            throws_tokens,
        },
        semi_token,
        trusted,
    }))
}

fn parse_extern_verbatim(
    cx: &mut Errors,
    tokens: &TokenStream,
    lang: Lang,
    trusted: bool,
    namespace: &Namespace,
) -> Result<Api> {
    |input: ParseStream| -> Result<Api> {
        let attrs = input.call(Attribute::parse_outer)?;
        let type_token: Token![type] = match input.parse()? {
            Some(type_token) => type_token,
            None => {
                let span = input.cursor().token_stream();
                return Err(Error::new_spanned(span, "unsupported foreign item"));
            }
        };
        let ident: Ident = input.parse()?;
        let generics: Generics = input.parse()?;
        let mut lifetimes = Punctuated::new();
        let mut has_unsupported_generic_param = false;
        for pair in generics.params.into_pairs() {
            let (param, punct) = pair.into_tuple();
            match param {
                GenericParam::Lifetime(param) => {
                    if !param.bounds.is_empty() && !has_unsupported_generic_param {
                        let msg = "lifetime parameter with bounds is not supported yet";
                        cx.error(&param, msg);
                        has_unsupported_generic_param = true;
                    }
                    lifetimes.push_value(param.lifetime);
                    if let Some(punct) = punct {
                        lifetimes.push_punct(punct);
                    }
                }
                GenericParam::Type(param) => {
                    if !has_unsupported_generic_param {
                        let msg = "extern type with generic type parameter is not supported yet";
                        cx.error(&param, msg);
                        has_unsupported_generic_param = true;
                    }
                }
                GenericParam::Const(param) => {
                    if !has_unsupported_generic_param {
                        let msg = "extern type with const generic parameter is not supported yet";
                        cx.error(&param, msg);
                        has_unsupported_generic_param = true;
                    }
                }
            }
        }
        let lifetimes = Lifetimes {
            lt_token: generics.lt_token,
            lifetimes,
            gt_token: generics.gt_token,
        };
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![=]) {
            // type Alias = crate::path::to::Type;
            parse_type_alias(
                cx, attrs, type_token, ident, lifetimes, input, lang, namespace,
            )
        } else if lookahead.peek(Token![:]) || lookahead.peek(Token![;]) {
            // type Opaque: Bound2 + Bound2;
            parse_extern_type_bounded(
                cx, attrs, type_token, ident, lifetimes, input, lang, trusted, namespace,
            )
        } else {
            Err(lookahead.error())
        }
    }
    .parse2(tokens.clone())
}

fn parse_type_alias(
    cx: &mut Errors,
    attrs: Vec<Attribute>,
    type_token: Token![type],
    ident: Ident,
    generics: Lifetimes,
    input: ParseStream,
    lang: Lang,
    namespace: &Namespace,
) -> Result<Api> {
    let eq_token: Token![=] = input.parse()?;
    let ty: RustType = input.parse()?;
    let semi_token: Token![;] = input.parse()?;

    let mut doc = Doc::new();
    let mut derives = Vec::new();
    let mut namespace = namespace.clone();
    let mut cxx_name = None;
    let mut rust_name = None;
    attrs::parse(
        cx,
        &attrs,
        attrs::Parser {
            doc: Some(&mut doc),
            derives: Some(&mut derives),
            namespace: Some(&mut namespace),
            cxx_name: Some(&mut cxx_name),
            rust_name: Some(&mut rust_name),
            ..Default::default()
        },
    );

    if lang == Lang::Rust {
        let span = quote!(#type_token #semi_token);
        let msg = "type alias in extern \"Rust\" block is not supported";
        return Err(Error::new_spanned(span, msg));
    }

    let name = pair(namespace, &ident, cxx_name, rust_name);

    Ok(Api::TypeAlias(TypeAlias {
        doc,
        derives,
        type_token,
        name,
        generics,
        eq_token,
        ty,
        semi_token,
    }))
}

fn parse_extern_type_bounded(
    cx: &mut Errors,
    attrs: Vec<Attribute>,
    type_token: Token![type],
    ident: Ident,
    generics: Lifetimes,
    input: ParseStream,
    lang: Lang,
    trusted: bool,
    namespace: &Namespace,
) -> Result<Api> {
    let mut bounds = Vec::new();
    let colon_token: Option<Token![:]> = input.parse()?;
    if colon_token.is_some() {
        loop {
            match input.parse()? {
                TypeParamBound::Trait(TraitBound {
                    paren_token: None,
                    modifier: TraitBoundModifier::None,
                    lifetimes: None,
                    path,
                }) if if let Some(derive) = path.get_ident().and_then(Derive::from) {
                    bounds.push(derive);
                    true
                } else {
                    false
                } => {}
                bound @ TypeParamBound::Trait(_) | bound @ TypeParamBound::Lifetime(_) => {
                    cx.error(bound, "unsupported trait");
                }
            }

            let lookahead = input.lookahead1();
            if lookahead.peek(Token![+]) {
                input.parse::<Token![+]>()?;
            } else if lookahead.peek(Token![;]) {
                break;
            } else {
                return Err(lookahead.error());
            }
        }
    }
    let semi_token: Token![;] = input.parse()?;

    let mut doc = Doc::new();
    let mut derives = Vec::new();
    let mut namespace = namespace.clone();
    let mut cxx_name = None;
    let mut rust_name = None;
    attrs::parse(
        cx,
        &attrs,
        attrs::Parser {
            doc: Some(&mut doc),
            derives: Some(&mut derives),
            namespace: Some(&mut namespace),
            cxx_name: Some(&mut cxx_name),
            rust_name: Some(&mut rust_name),
            ..Default::default()
        },
    );

    let name = pair(namespace, &ident, cxx_name, rust_name);

    Ok(match lang {
        Lang::Cxx => Api::CxxType,
        Lang::Rust => Api::RustType,
    }(ExternType {
        lang,
        doc,
        derives,
        type_token,
        name,
        generics,
        colon_token,
        bounds,
        semi_token,
        trusted,
    }))
}

fn parse_impl(imp: ItemImpl) -> Result<Api> {
    if !imp.items.is_empty() {
        let mut span = Group::new(Delimiter::Brace, TokenStream::new());
        span.set_span(imp.brace_token.span);
        return Err(Error::new_spanned(span, "expected an empty impl block"));
    }

    if let Some((bang, path, for_token)) = &imp.trait_ {
        let self_ty = &imp.self_ty;
        let span = quote!(#bang #path #for_token #self_ty);
        return Err(Error::new_spanned(
            span,
            "unexpected impl, expected something like `impl UniquePtr<T> {}`",
        ));
    }

    let generics = &imp.generics;
    if !generics.params.is_empty() || generics.where_clause.is_some() {
        return Err(Error::new_spanned(
            imp,
            "generic parameters on an impl is not supported",
        ));
    }

    let mut negative_token = None;
    let mut self_ty = *imp.self_ty;
    if let RustType::Verbatim(ty) = &self_ty {
        let mut iter = ty.clone().into_iter();
        if let Some(TokenTree::Punct(punct)) = iter.next() {
            if punct.as_char() == '!' {
                let ty = iter.collect::<TokenStream>();
                if !ty.is_empty() {
                    negative_token = Some(Token![!](punct.span()));
                    self_ty = syn::parse2(ty)?;
                }
            }
        }
    }

    let impl_token = imp.impl_token;
    let negative = negative_token.is_some();
    let ty = parse_type(&self_ty)?;
    let brace_token = imp.brace_token;

    Ok(Api::Impl(Impl {
        impl_token,
        negative,
        ty,
        brace_token,
        negative_token,
    }))
}

fn parse_include(input: ParseStream) -> Result<Include> {
    if input.peek(LitStr) {
        let lit: LitStr = input.parse()?;
        let span = lit.span();
        return Ok(Include {
            path: lit.value(),
            kind: IncludeKind::Quoted,
            begin_span: span,
            end_span: span,
        });
    }

    if input.peek(Token![<]) {
        let mut path = String::new();

        let langle: Token![<] = input.parse()?;
        while !input.is_empty() && !input.peek(Token![>]) {
            let token: TokenTree = input.parse()?;
            match token {
                TokenTree::Ident(token) => path += &token.to_string(),
                TokenTree::Literal(token)
                    if token
                        .to_string()
                        .starts_with(|ch: char| ch.is_ascii_digit()) =>
                {
                    path += &token.to_string();
                }
                TokenTree::Punct(token) => path.push(token.as_char()),
                _ => return Err(Error::new(token.span(), "unexpected token in include path")),
            }
        }
        let rangle: Token![>] = input.parse()?;

        return Ok(Include {
            path,
            kind: IncludeKind::Bracketed,
            begin_span: langle.span,
            end_span: rangle.span,
        });
    }

    Err(input.error("expected \"quoted/path/to\" or <bracketed/path/to>"))
}

fn parse_type(ty: &RustType) -> Result<Type> {
    match ty {
        RustType::Reference(ty) => parse_type_reference(ty),
        RustType::Path(ty) => parse_type_path(ty),
        RustType::Array(ty) => parse_type_array(ty),
        RustType::BareFn(ty) => parse_type_fn(ty),
        RustType::Tuple(ty) if ty.elems.is_empty() => Ok(Type::Void(ty.paren_token.span)),
        _ => Err(Error::new_spanned(ty, "unsupported type")),
    }
}

fn parse_type_reference(ty: &TypeReference) -> Result<Type> {
    let ampersand = ty.and_token;
    let lifetime = ty.lifetime.clone();
    let mutable = ty.mutability.is_some();
    let mutability = ty.mutability;

    if let RustType::Slice(slice) = ty.elem.as_ref() {
        let inner = parse_type(&slice.elem)?;
        let bracket = slice.bracket_token;
        return Ok(Type::SliceRef(Box::new(SliceRef {
            ampersand,
            lifetime,
            mutable,
            bracket,
            inner,
            mutability,
        })));
    }

    let inner = parse_type(&ty.elem)?;
    let pinned = false;
    let pin_tokens = None;

    Ok(match &inner {
        Type::Ident(ident) if ident.rust == "str" => {
            if ty.mutability.is_some() {
                return Err(Error::new_spanned(ty, "unsupported type"));
            } else {
                Type::Str
            }
        }
        _ => Type::Ref,
    }(Box::new(Ref {
        pinned,
        ampersand,
        lifetime,
        mutable,
        inner,
        pin_tokens,
        mutability,
    })))
}

fn parse_type_path(ty: &TypePath) -> Result<Type> {
    let path = &ty.path;
    if ty.qself.is_none() && path.leading_colon.is_none() && path.segments.len() == 1 {
        let segment = &path.segments[0];
        let ident = segment.ident.clone();
        match &segment.arguments {
            PathArguments::None => return Ok(Type::Ident(RustName::new(ident))),
            PathArguments::AngleBracketed(generic) => {
                if ident == "UniquePtr" && generic.args.len() == 1 {
                    if let GenericArgument::Type(arg) = &generic.args[0] {
                        let inner = parse_type(arg)?;
                        return Ok(Type::UniquePtr(Box::new(Ty1 {
                            name: ident,
                            langle: generic.lt_token,
                            inner,
                            rangle: generic.gt_token,
                        })));
                    }
                } else if ident == "SharedPtr" && generic.args.len() == 1 {
                    if let GenericArgument::Type(arg) = &generic.args[0] {
                        let inner = parse_type(arg)?;
                        return Ok(Type::SharedPtr(Box::new(Ty1 {
                            name: ident,
                            langle: generic.lt_token,
                            inner,
                            rangle: generic.gt_token,
                        })));
                    }
                } else if ident == "WeakPtr" && generic.args.len() == 1 {
                    if let GenericArgument::Type(arg) = &generic.args[0] {
                        let inner = parse_type(arg)?;
                        return Ok(Type::WeakPtr(Box::new(Ty1 {
                            name: ident,
                            langle: generic.lt_token,
                            inner,
                            rangle: generic.gt_token,
                        })));
                    }
                } else if ident == "CxxVector" && generic.args.len() == 1 {
                    if let GenericArgument::Type(arg) = &generic.args[0] {
                        let inner = parse_type(arg)?;
                        return Ok(Type::CxxVector(Box::new(Ty1 {
                            name: ident,
                            langle: generic.lt_token,
                            inner,
                            rangle: generic.gt_token,
                        })));
                    }
                } else if ident == "Box" && generic.args.len() == 1 {
                    if let GenericArgument::Type(arg) = &generic.args[0] {
                        let inner = parse_type(arg)?;
                        return Ok(Type::RustBox(Box::new(Ty1 {
                            name: ident,
                            langle: generic.lt_token,
                            inner,
                            rangle: generic.gt_token,
                        })));
                    }
                } else if ident == "Vec" && generic.args.len() == 1 {
                    if let GenericArgument::Type(arg) = &generic.args[0] {
                        let inner = parse_type(arg)?;
                        return Ok(Type::RustVec(Box::new(Ty1 {
                            name: ident,
                            langle: generic.lt_token,
                            inner,
                            rangle: generic.gt_token,
                        })));
                    }
                } else if ident == "Pin" && generic.args.len() == 1 {
                    if let GenericArgument::Type(arg) = &generic.args[0] {
                        let inner = parse_type(arg)?;
                        let pin_token = kw::Pin(ident.span());
                        if let Type::Ref(mut inner) = inner {
                            inner.pinned = true;
                            inner.pin_tokens =
                                Some((pin_token, generic.lt_token, generic.gt_token));
                            return Ok(Type::Ref(inner));
                        }
                    }
                }
            }
            PathArguments::Parenthesized(_) => {}
        }
    }

    Err(Error::new_spanned(ty, "unsupported type"))
}

fn parse_type_array(ty: &TypeArray) -> Result<Type> {
    let inner = parse_type(&ty.elem)?;

    let len_expr = if let Expr::Lit(lit) = &ty.len {
        lit
    } else {
        let msg = "unsupported expression, array length must be an integer literal";
        return Err(Error::new_spanned(&ty.len, msg));
    };

    let len_token = if let Lit::Int(int) = &len_expr.lit {
        int.clone()
    } else {
        let msg = "array length must be an integer literal";
        return Err(Error::new_spanned(len_expr, msg));
    };

    let len = len_token.base10_parse::<usize>()?;
    if len == 0 {
        let msg = "array with zero size is not supported";
        return Err(Error::new_spanned(ty, msg));
    }

    let bracket = ty.bracket_token;
    let semi_token = ty.semi_token;

    Ok(Type::Array(Box::new(Array {
        bracket,
        inner,
        semi_token,
        len,
        len_token,
    })))
}

fn parse_type_fn(ty: &TypeBareFn) -> Result<Type> {
    if ty.lifetimes.is_some() {
        return Err(Error::new_spanned(
            ty,
            "function pointer with lifetime parameters is not supported yet",
        ));
    }

    if ty.variadic.is_some() {
        return Err(Error::new_spanned(
            ty,
            "variadic function pointer is not supported yet",
        ));
    }

    let args = ty
        .inputs
        .iter()
        .enumerate()
        .map(|(i, arg)| {
            let ty = parse_type(&arg.ty)?;
            let ident = match &arg.name {
                Some(ident) => ident.0.clone(),
                None => format_ident!("arg{}", i),
            };
            Ok(Var { ident, ty })
        })
        .collect::<Result<_>>()?;

    let mut throws_tokens = None;
    let ret = parse_return_type(&ty.output, &mut throws_tokens)?;
    let throws = throws_tokens.is_some();

    let unsafety = ty.unsafety;
    let fn_token = ty.fn_token;
    let generics = Generics::default();
    let receiver = None;
    let paren_token = ty.paren_token;

    Ok(Type::Fn(Box::new(Signature {
        unsafety,
        fn_token,
        generics,
        receiver,
        args,
        ret,
        throws,
        paren_token,
        throws_tokens,
    })))
}

fn parse_return_type(
    ty: &ReturnType,
    throws_tokens: &mut Option<(kw::Result, Token![<], Token![>])>,
) -> Result<Option<Type>> {
    let mut ret = match ty {
        ReturnType::Default => return Ok(None),
        ReturnType::Type(_, ret) => ret.as_ref(),
    };

    if let RustType::Path(ty) = ret {
        let path = &ty.path;
        if ty.qself.is_none() && path.leading_colon.is_none() && path.segments.len() == 1 {
            let segment = &path.segments[0];
            let ident = segment.ident.clone();
            if let PathArguments::AngleBracketed(generic) = &segment.arguments {
                if ident == "Result" && generic.args.len() == 1 {
                    if let GenericArgument::Type(arg) = &generic.args[0] {
                        ret = arg;
                        *throws_tokens =
                            Some((kw::Result(ident.span()), generic.lt_token, generic.gt_token));
                    }
                }
            }
        }
    }

    match parse_type(ret)? {
        Type::Void(_) => Ok(None),
        ty => Ok(Some(ty)),
    }
}

fn pair(namespace: Namespace, default: &Ident, cxx: Option<Ident>, rust: Option<Ident>) -> Pair {
    let default = || default.clone();
    Pair {
        namespace,
        cxx: cxx.unwrap_or_else(default),
        rust: rust.unwrap_or_else(default),
    }
}
