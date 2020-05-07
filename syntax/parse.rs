use crate::syntax::report::Errors;
use crate::syntax::Atom::*;
use crate::syntax::{
    attrs, error, Api, Doc, Enum, ExternFn, ExternType, Lang, Receiver, Ref, Signature, Slice,
    Struct, Ty1, Type, Var, Variant,
};
use quote::{format_ident, quote};
use std::collections::HashSet;
use std::u32;
use syn::punctuated::Punctuated;
use syn::{
    Abi, Error, Expr, ExprLit, Fields, FnArg, ForeignItem, ForeignItemFn, ForeignItemType,
    GenericArgument, Ident, Item, ItemEnum, ItemForeignMod, ItemStruct, Lit, Pat, PathArguments,
    Result, ReturnType, Token, Type as RustType, TypeBareFn, TypePath, TypeReference, TypeSlice,
    Variant as RustVariant,
};

pub mod kw {
    syn::custom_keyword!(Result);
}

pub fn parse_items(cx: &mut Errors, items: Vec<Item>) -> Vec<Api> {
    let mut apis = Vec::new();
    for item in items {
        match item {
            Item::Struct(item) => match parse_struct(item) {
                Ok(strct) => apis.push(strct),
                Err(err) => cx.push(err),
            },
            Item::Enum(item) => match parse_enum(item) {
                Ok(enm) => apis.push(enm),
                Err(err) => cx.push(err),
            },
            Item::ForeignMod(foreign_mod) => parse_foreign_mod(cx, foreign_mod, &mut apis),
            Item::Use(item) => cx.error(item, error::USE_NOT_ALLOWED),
            _ => cx.error(item, "unsupported item"),
        }
    }
    apis
}

fn parse_struct(item: ItemStruct) -> Result<Api> {
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
    attrs::parse(&item.attrs, &mut doc, Some(&mut derives))?;

    let fields = match item.fields {
        Fields::Named(fields) => fields,
        Fields::Unit => return Err(Error::new_spanned(item, "unit structs are not supported")),
        Fields::Unnamed(_) => {
            return Err(Error::new_spanned(item, "tuple structs are not supported"))
        }
    };

    Ok(Api::Struct(Struct {
        doc,
        derives,
        struct_token: item.struct_token,
        ident: item.ident,
        brace_token: fields.brace_token,
        fields: fields
            .named
            .into_iter()
            .map(|field| {
                Ok(Var {
                    ident: field.ident.unwrap(),
                    ty: parse_type(&field.ty)?,
                })
            })
            .collect::<Result<_>>()?,
    }))
}

fn parse_enum(item: ItemEnum) -> Result<Api> {
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

    let doc = attrs::parse_doc(&item.attrs)?;

    let mut variants = Vec::new();
    let mut discriminants = HashSet::new();
    let mut prev_discriminant = None;
    for variant in item.variants {
        match variant.fields {
            Fields::Unit => {}
            _ => {
                return Err(Error::new_spanned(
                    variant,
                    "enums with data are not supported yet",
                ));
            }
        }
        if variant.discriminant.is_none() && prev_discriminant == Some(u32::MAX) {
            let msg = format!("discriminant overflow on value after {}", u32::MAX);
            return Err(Error::new_spanned(variant, msg));
        }
        let discriminant =
            parse_discriminant(&variant)?.unwrap_or_else(|| prev_discriminant.map_or(0, |n| n + 1));
        if !discriminants.insert(discriminant) {
            let msg = format!("discriminant value `{}` already exists", discriminant);
            return Err(Error::new_spanned(variant, msg));
        }
        variants.push(Variant {
            ident: variant.ident,
            discriminant,
        });
        prev_discriminant = Some(discriminant);
    }

    Ok(Api::Enum(Enum {
        doc,
        enum_token: item.enum_token,
        ident: item.ident,
        brace_token: item.brace_token,
        variants,
    }))
}

fn parse_discriminant(variant: &RustVariant) -> Result<Option<u32>> {
    match &variant.discriminant {
        None => Ok(None),
        Some((
            _,
            Expr::Lit(ExprLit {
                lit: Lit::Int(n), ..
            }),
        )) => match n.base10_parse() {
            Ok(val) => Ok(Some(val)),
            Err(_) => Err(Error::new_spanned(
                variant,
                "cannot parse enum discriminant as an integer",
            )),
        },
        _ => Err(Error::new_spanned(
            variant,
            "enums with non-integer literal discriminants are not supported yet",
        )),
    }
}

fn parse_foreign_mod(cx: &mut Errors, foreign_mod: ItemForeignMod, out: &mut Vec<Api>) {
    let lang = match parse_lang(foreign_mod.abi) {
        Ok(lang) => lang,
        Err(err) => return cx.push(err),
    };

    let mut items = Vec::new();
    for foreign in &foreign_mod.items {
        match foreign {
            ForeignItem::Type(foreign) => match parse_extern_type(foreign, lang) {
                Ok(ety) => items.push(ety),
                Err(err) => cx.push(err),
            },
            ForeignItem::Fn(foreign) => match parse_extern_fn(foreign, lang) {
                Ok(efn) => items.push(efn),
                Err(err) => cx.push(err),
            },
            ForeignItem::Macro(foreign) if foreign.mac.path.is_ident("include") => {
                match foreign.mac.parse_body() {
                    Ok(include) => items.push(Api::Include(include)),
                    Err(err) => cx.push(err),
                }
            }
            _ => cx.error(foreign, "unsupported foreign item"),
        }
    }

    let mut types = items.iter().filter_map(|item| match item {
        Api::CxxType(ty) | Api::RustType(ty) => Some(ty),
        _ => None,
    });
    if let (Some(single_type), None) = (types.next(), types.next()) {
        let single_type = single_type.ident.clone();
        for item in &mut items {
            if let Api::CxxFunction(efn) | Api::RustFunction(efn) = item {
                if let Some(receiver) = &mut efn.receiver {
                    if receiver.ty == "Self" {
                        receiver.ty = single_type.clone();
                    }
                }
            }
        }
    }

    out.extend(items);
}

fn parse_lang(abi: Abi) -> Result<Lang> {
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

fn parse_extern_type(foreign_type: &ForeignItemType, lang: Lang) -> Result<Api> {
    let doc = attrs::parse_doc(&foreign_type.attrs)?;
    let type_token = foreign_type.type_token;
    let ident = foreign_type.ident.clone();
    let api_type = match lang {
        Lang::Cxx => Api::CxxType,
        Lang::Rust => Api::RustType,
    };
    Ok(api_type(ExternType {
        doc,
        type_token,
        ident,
    }))
}

fn parse_extern_fn(foreign_fn: &ForeignItemFn, lang: Lang) -> Result<Api> {
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
                        ty: Token![Self](arg.self_token.span).into(),
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
                            ampersand: reference.ampersand,
                            lifetime: reference.lifetime,
                            mutability: reference.mutability,
                            var: Token![self](ident.span()),
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
    let ret = parse_return_type(&foreign_fn.sig.output, &mut throws_tokens)?;
    let throws = throws_tokens.is_some();
    let doc = attrs::parse_doc(&foreign_fn.attrs)?;
    let fn_token = foreign_fn.sig.fn_token;
    let ident = foreign_fn.sig.ident.clone();
    let paren_token = foreign_fn.sig.paren_token;
    let semi_token = foreign_fn.semi_token;
    let api_function = match lang {
        Lang::Cxx => Api::CxxFunction,
        Lang::Rust => Api::RustFunction,
    };

    Ok(api_function(ExternFn {
        lang,
        doc,
        ident,
        sig: Signature {
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

fn parse_type(ty: &RustType) -> Result<Type> {
    match ty {
        RustType::Reference(ty) => parse_type_reference(ty),
        RustType::Path(ty) => parse_type_path(ty),
        RustType::Slice(ty) => parse_type_slice(ty),
        RustType::BareFn(ty) => parse_type_fn(ty),
        RustType::Tuple(ty) if ty.elems.is_empty() => Ok(Type::Void(ty.paren_token.span)),
        _ => Err(Error::new_spanned(ty, "unsupported type")),
    }
}

fn parse_type_reference(ty: &TypeReference) -> Result<Type> {
    let inner = parse_type(&ty.elem)?;
    let which = match &inner {
        Type::Ident(ident) if ident == "str" => {
            if ty.mutability.is_some() {
                return Err(Error::new_spanned(ty, "unsupported type"));
            } else {
                Type::Str
            }
        }
        Type::Slice(slice) => match &slice.inner {
            Type::Ident(ident) if ident == U8 && ty.mutability.is_none() => Type::SliceRefU8,
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

fn parse_type_path(ty: &TypePath) -> Result<Type> {
    let path = &ty.path;
    if ty.qself.is_none() && path.leading_colon.is_none() && path.segments.len() == 1 {
        let segment = &path.segments[0];
        let ident = segment.ident.clone();
        match &segment.arguments {
            PathArguments::None => return Ok(Type::Ident(ident)),
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
                }
            }
            PathArguments::Parenthesized(_) => {}
        }
    }
    Err(Error::new_spanned(ty, "unsupported type"))
}

fn parse_type_slice(ty: &TypeSlice) -> Result<Type> {
    let inner = parse_type(&ty.elem)?;
    Ok(Type::Slice(Box::new(Slice {
        bracket: ty.bracket_token,
        inner,
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
                None => format_ident!("_{}", i),
            };
            Ok(Var { ident, ty })
        })
        .collect::<Result<_>>()?;
    let mut throws_tokens = None;
    let ret = parse_return_type(&ty.output, &mut throws_tokens)?;
    let throws = throws_tokens.is_some();
    Ok(Type::Fn(Box::new(Signature {
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
