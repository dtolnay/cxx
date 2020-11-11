use crate::syntax::discriminant::DiscriminantSet;
use crate::syntax::file::{Item, ItemForeignMod};
use crate::syntax::report::Errors;
use crate::syntax::Atom::*;
use crate::syntax::{
    attrs, error, Api, Doc, Enum, ExternFn, ExternType, Impl, Include, IncludeKind, Lang,
    Namespace, Pair, Receiver, Ref, ResolvableName, Signature, Slice, Struct, Ty1, Type, TypeAlias,
    Var, Variant,
};
use proc_macro2::{Delimiter, Group, TokenStream, TokenTree};
use quote::{format_ident, quote, quote_spanned};
use syn::parse::{ParseStream, Parser};
use syn::punctuated::Punctuated;
use syn::{
    Abi, Attribute, Error, Fields, FnArg, ForeignItem, ForeignItemFn, ForeignItemType,
    GenericArgument, Ident, ItemEnum, ItemImpl, ItemStruct, LitStr, Pat, PathArguments, Result,
    ReturnType, Token, Type as RustType, TypeBareFn, TypePath, TypeReference, TypeSlice,
};

pub mod kw {
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
            Item::Impl(item) => match parse_impl(item, namespace) {
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

    let mut doc = Doc::new();
    let mut derives = Vec::new();
    let mut namespace = namespace.clone();
    attrs::parse(
        cx,
        &item.attrs,
        attrs::Parser {
            doc: Some(&mut doc),
            derives: Some(&mut derives),
            namespace: Some(&mut namespace),
            ..Default::default()
        },
    );

    let named_fields = match item.fields {
        Fields::Named(fields) => fields,
        Fields::Unit => return Err(Error::new_spanned(item, "unit structs are not supported")),
        Fields::Unnamed(_) => {
            return Err(Error::new_spanned(item, "tuple structs are not supported"));
        }
    };

    let fields = named_fields
        .named
        .into_iter()
        .map(|field| {
            Ok(Var {
                ident: field.ident.unwrap(),
                ty: parse_type(&field.ty, &namespace)?,
            })
        })
        .collect::<Result<_>>()?;

    Ok(Api::Struct(Struct {
        doc,
        derives,
        struct_token: item.struct_token,
        name: Pair::new(namespace, item.ident),
        brace_token: named_fields.brace_token,
        fields,
    }))
}

fn parse_enum(cx: &mut Errors, item: ItemEnum, namespace: &Namespace) -> Result<Api> {
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

    let mut doc = Doc::new();
    let mut repr = None;
    let mut namespace = namespace.clone();
    attrs::parse(
        cx,
        &item.attrs,
        attrs::Parser {
            doc: Some(&mut doc),
            repr: Some(&mut repr),
            namespace: Some(&mut namespace),
            ..Default::default()
        },
    );

    let mut variants = Vec::new();
    let mut discriminants = DiscriminantSet::new(repr);
    for variant in item.variants {
        match variant.fields {
            Fields::Unit => {}
            _ => {
                cx.error(variant, "enums with data are not supported yet");
                break;
            }
        }
        let expr = variant.discriminant.as_ref().map(|(_, expr)| expr);
        let try_discriminant = match &expr {
            Some(lit) => discriminants.insert(lit),
            None => discriminants.insert_next(),
        };
        let discriminant = match try_discriminant {
            Ok(discriminant) => discriminant,
            Err(err) => {
                cx.error(variant, err);
                break;
            }
        };
        let expr = variant.discriminant.map(|(_, expr)| expr);
        variants.push(Variant {
            ident: variant.ident,
            discriminant,
            expr,
        });
    }

    let enum_token = item.enum_token;
    let brace_token = item.brace_token;

    let mut repr = U8;
    match discriminants.inferred_repr() {
        Ok(inferred) => repr = inferred,
        Err(err) => {
            let span = quote_spanned!(brace_token.span=> #enum_token {});
            cx.error(span, err);
            variants.clear();
        }
    }

    Ok(Api::Enum(Enum {
        doc,
        enum_token,
        name: Pair::new(namespace, item.ident),
        brace_token,
        variants,
        repr,
    }))
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
                let abi = foreign_mod.abi;
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
                match parse_extern_type(cx, foreign, lang, trusted, &namespace) {
                    Ok(ety) => items.push(ety),
                    Err(err) => cx.push(err),
                }
            }
            ForeignItem::Fn(foreign) => match parse_extern_fn(cx, foreign, lang, &namespace) {
                Ok(efn) => items.push(efn),
                Err(err) => cx.push(err),
            },
            ForeignItem::Macro(foreign) if foreign.mac.path.is_ident("include") => {
                match foreign.mac.parse_body_with(parse_include) {
                    Ok(include) => items.push(Api::Include(include)),
                    Err(err) => cx.push(err),
                }
            }
            ForeignItem::Verbatim(tokens) => {
                match parse_extern_verbatim(cx, tokens, lang, &namespace) {
                    Ok(api) => items.push(api),
                    Err(err) => cx.push(err),
                }
            }
            _ => cx.error(foreign, "unsupported foreign item"),
        }
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
                    if receiver.ty.is_self() {
                        receiver.ty = ResolvableName::new(single_type.rust.clone());
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
                "ABI name is required, extern \"C\" or extern \"Rust\"",
            ));
        }
    };
    match name.value().as_str() {
        "C" | "C++" => Ok(Lang::Cxx),
        "Rust" => Ok(Lang::Rust),
        _ => Err(Error::new_spanned(abi, "unrecognized ABI")),
    }
}

fn parse_extern_type(
    cx: &mut Errors,
    foreign_type: &ForeignItemType,
    lang: Lang,
    trusted: bool,
    namespace: &Namespace,
) -> Result<Api> {
    let mut doc = Doc::new();
    let mut namespace = namespace.clone();
    attrs::parse(
        cx,
        &foreign_type.attrs,
        attrs::Parser {
            doc: Some(&mut doc),
            namespace: Some(&mut namespace),
            ..Default::default()
        },
    );
    let type_token = foreign_type.type_token;
    let ident = foreign_type.ident.clone();
    let semi_token = foreign_type.semi_token;
    let api_type = match lang {
        Lang::Cxx => Api::CxxType,
        Lang::Rust => Api::RustType,
    };
    Ok(api_type(ExternType {
        doc,
        type_token,
        name: Pair::new(namespace, ident),
        semi_token,
        trusted,
    }))
}

fn parse_extern_fn(
    cx: &mut Errors,
    foreign_fn: &ForeignItemFn,
    lang: Lang,
    namespace: &Namespace,
) -> Result<Api> {
    let generics = &foreign_fn.sig.generics;
    if !generics.params.is_empty() || generics.where_clause.is_some() {
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

    let mut doc = Doc::new();
    let mut cxx_name = None;
    let mut rust_name = None;
    let mut namespace = namespace.clone();
    attrs::parse(
        cx,
        &foreign_fn.attrs,
        attrs::Parser {
            doc: Some(&mut doc),
            cxx_name: Some(&mut cxx_name),
            rust_name: Some(&mut rust_name),
            namespace: Some(&mut namespace),
            ..Default::default()
        },
    );

    let mut receiver = None;
    let mut args = Punctuated::new();
    for arg in foreign_fn.sig.inputs.pairs() {
        let (arg, comma) = arg.into_tuple();
        match arg {
            FnArg::Receiver(arg) => {
                if let Some((ampersand, lifetime)) = &arg.reference {
                    receiver = Some(Receiver {
                        ampersand: *ampersand,
                        lifetime: lifetime.clone(),
                        mutability: arg.mutability,
                        var: arg.self_token,
                        ty: ResolvableName::make_self(arg.self_token.span),
                        shorthand: true,
                    });
                    continue;
                }
                return Err(Error::new_spanned(arg, "unsupported signature"));
            }
            FnArg::Typed(arg) => {
                let ident = match arg.pat.as_ref() {
                    Pat::Ident(pat) => pat.ident.clone(),
                    Pat::Wild(pat) => {
                        Ident::new(&format!("_{}", args.len()), pat.underscore_token.span)
                    }
                    _ => return Err(Error::new_spanned(arg, "unsupported signature")),
                };
                let ty = parse_type(&arg.ty, &namespace)?;
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
                            ampersand: reference.ampersand,
                            lifetime: reference.lifetime,
                            mutability: reference.mutability,
                            var: Token![self](ident.rust.span()),
                            ty: ident,
                            shorthand: false,
                        });
                        continue;
                    }
                }
                return Err(Error::new_spanned(arg, "unsupported method receiver"));
            }
        }
    }

    let mut throws_tokens = None;
    let ret = parse_return_type(&foreign_fn.sig.output, &mut throws_tokens, &namespace)?;
    let throws = throws_tokens.is_some();
    let unsafety = foreign_fn.sig.unsafety;
    let fn_token = foreign_fn.sig.fn_token;
    let name = Pair::new_from_differing_names(
        namespace,
        cxx_name.unwrap_or(foreign_fn.sig.ident.clone()),
        rust_name.unwrap_or(foreign_fn.sig.ident.clone()),
    );
    let paren_token = foreign_fn.sig.paren_token;
    let semi_token = foreign_fn.semi_token;
    let api_function = match lang {
        Lang::Cxx => Api::CxxFunction,
        Lang::Rust => Api::RustFunction,
    };

    Ok(api_function(ExternFn {
        lang,
        doc,
        name,
        sig: Signature {
            unsafety,
            fn_token,
            receiver,
            args,
            ret,
            throws,
            paren_token,
            throws_tokens,
        },
        semi_token,
    }))
}

fn parse_extern_verbatim(
    cx: &mut Errors,
    tokens: &TokenStream,
    lang: Lang,
    namespace: &Namespace,
) -> Result<Api> {
    // type Alias = crate::path::to::Type;
    let parse = |input: ParseStream| -> Result<TypeAlias> {
        let attrs = input.call(Attribute::parse_outer)?;
        let type_token: Token![type] = match input.parse()? {
            Some(type_token) => type_token,
            None => {
                let span = input.cursor().token_stream();
                return Err(Error::new_spanned(span, "unsupported foreign item"));
            }
        };
        let ident: Ident = input.parse()?;
        let eq_token: Token![=] = input.parse()?;
        let ty: RustType = input.parse()?;
        let semi_token: Token![;] = input.parse()?;
        let mut doc = Doc::new();
        let mut namespace = namespace.clone();
        attrs::parse(
            cx,
            &attrs,
            attrs::Parser {
                doc: Some(&mut doc),
                namespace: Some(&mut namespace),
                ..Default::default()
            },
        );

        Ok(TypeAlias {
            doc,
            type_token,
            name: Pair::new(namespace, ident),
            eq_token,
            ty,
            semi_token,
        })
    };

    let type_alias = parse.parse2(tokens.clone())?;
    match lang {
        Lang::Cxx => Ok(Api::TypeAlias(type_alias)),
        Lang::Rust => {
            let (type_token, semi_token) = (type_alias.type_token, type_alias.semi_token);
            let span = quote!(#type_token #semi_token);
            let msg = "type alias in extern \"Rust\" block is not supported";
            Err(Error::new_spanned(span, msg))
        }
    }
}

fn parse_impl(imp: ItemImpl, namespace: &Namespace) -> Result<Api> {
    if !imp.items.is_empty() {
        let mut span = Group::new(Delimiter::Brace, TokenStream::new());
        span.set_span(imp.brace_token.span);
        return Err(Error::new_spanned(span, "expected an empty impl block"));
    }

    let self_ty = &imp.self_ty;
    if let Some((bang, path, for_token)) = &imp.trait_ {
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

    Ok(Api::Impl(Impl {
        impl_token: imp.impl_token,
        ty: parse_type(&self_ty, namespace)?,
        brace_token: imp.brace_token,
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

fn parse_type(ty: &RustType, namespace: &Namespace) -> Result<Type> {
    match ty {
        RustType::Reference(ty) => parse_type_reference(ty, namespace),
        RustType::Path(ty) => parse_type_path(ty, namespace),
        RustType::Slice(ty) => parse_type_slice(ty, namespace),
        RustType::BareFn(ty) => parse_type_fn(ty, namespace),
        RustType::Tuple(ty) if ty.elems.is_empty() => Ok(Type::Void(ty.paren_token.span)),
        _ => Err(Error::new_spanned(ty, "unsupported type")),
    }
}

fn parse_type_reference(ty: &TypeReference, namespace: &Namespace) -> Result<Type> {
    let inner = parse_type(&ty.elem, namespace)?;
    let which = match &inner {
        Type::Ident(ident) if ident.rust == "str" => {
            if ty.mutability.is_some() {
                return Err(Error::new_spanned(ty, "unsupported type"));
            } else {
                Type::Str
            }
        }
        Type::Slice(slice) => match &slice.inner {
            Type::Ident(ident) if ident.rust == U8 && ty.mutability.is_none() => Type::SliceRefU8,
            _ => Type::Ref,
        },
        _ => Type::Ref,
    };
    Ok(which(Box::new(Ref {
        ampersand: ty.and_token,
        lifetime: ty.lifetime.clone(),
        mutability: ty.mutability,
        inner,
    })))
}

fn parse_type_path(ty: &TypePath, namespace: &Namespace) -> Result<Type> {
    let path = &ty.path;
    if ty.qself.is_none() && path.leading_colon.is_none() && path.segments.len() == 1 {
        let segment = &path.segments[0];
        let ident = segment.ident.clone();
        match &segment.arguments {
            PathArguments::None => return Ok(Type::Ident(ResolvableName::new(ident))),
            PathArguments::AngleBracketed(generic) => {
                if ident == "UniquePtr" && generic.args.len() == 1 {
                    if let GenericArgument::Type(arg) = &generic.args[0] {
                        let inner = parse_type(arg, namespace)?;
                        return Ok(Type::UniquePtr(Box::new(Ty1 {
                            name: ident,
                            langle: generic.lt_token,
                            inner,
                            rangle: generic.gt_token,
                        })));
                    }
                } else if ident == "CxxVector" && generic.args.len() == 1 {
                    if let GenericArgument::Type(arg) = &generic.args[0] {
                        let inner = parse_type(arg, namespace)?;
                        return Ok(Type::CxxVector(Box::new(Ty1 {
                            name: ident,
                            langle: generic.lt_token,
                            inner,
                            rangle: generic.gt_token,
                        })));
                    }
                } else if ident == "Box" && generic.args.len() == 1 {
                    if let GenericArgument::Type(arg) = &generic.args[0] {
                        let inner = parse_type(arg, namespace)?;
                        return Ok(Type::RustBox(Box::new(Ty1 {
                            name: ident,
                            langle: generic.lt_token,
                            inner,
                            rangle: generic.gt_token,
                        })));
                    }
                } else if ident == "Vec" && generic.args.len() == 1 {
                    if let GenericArgument::Type(arg) = &generic.args[0] {
                        let inner = parse_type(arg, namespace)?;
                        return Ok(Type::RustVec(Box::new(Ty1 {
                            name: ident,
                            langle: generic.lt_token,
                            inner,
                            rangle: generic.gt_token,
                        })));
                    }
                }
            }
            PathArguments::Parenthesized(_) => {}
        }
    }
    Err(Error::new_spanned(ty, "unsupported type"))
}

fn parse_type_slice(ty: &TypeSlice, namespace: &Namespace) -> Result<Type> {
    let inner = parse_type(&ty.elem, namespace)?;
    Ok(Type::Slice(Box::new(Slice {
        bracket: ty.bracket_token,
        inner,
    })))
}

fn parse_type_fn(ty: &TypeBareFn, namespace: &Namespace) -> Result<Type> {
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
            let ty = parse_type(&arg.ty, namespace)?;
            let ident = match &arg.name {
                Some(ident) => ident.0.clone(),
                None => format_ident!("_{}", i),
            };
            Ok(Var { ident, ty })
        })
        .collect::<Result<_>>()?;
    let mut throws_tokens = None;
    let ret = parse_return_type(&ty.output, &mut throws_tokens, namespace)?;
    let throws = throws_tokens.is_some();
    Ok(Type::Fn(Box::new(Signature {
        unsafety: ty.unsafety,
        fn_token: ty.fn_token,
        receiver: None,
        args,
        ret,
        throws,
        paren_token: ty.paren_token,
        throws_tokens,
    })))
}

fn parse_return_type(
    ty: &ReturnType,
    throws_tokens: &mut Option<(kw::Result, Token![<], Token![>])>,
    namespace: &Namespace,
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
    match parse_type(ret, namespace)? {
        Type::Void(_) => Ok(None),
        ty => Ok(Some(ty)),
    }
}
