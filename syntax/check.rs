use crate::syntax::atom::Atom::{self, *};
use crate::syntax::{
    error, ident, Api, ExternFn, Lang, Receiver, Ref, Slice, Struct, Ty1, Type, Types,
};
use proc_macro2::{Delimiter, Group, Ident, TokenStream};
use quote::{quote, ToTokens};
use std::fmt::Display;
use syn::{Error, Result};

struct Check<'a> {
    apis: &'a [Api],
    types: &'a Types<'a>,
    errors: &'a mut Vec<Error>,
}

pub(crate) fn typecheck(apis: &[Api], types: &Types) -> Result<()> {
    let mut errors = Vec::new();
    let mut cx = Check {
        apis,
        types,
        errors: &mut errors,
    };
    do_typecheck(&mut cx);
    combine_errors(errors)
}

fn do_typecheck(cx: &mut Check) {
    for ty in cx.types {
        match ty {
            Type::Ident(ident) => check_type_ident(cx, ident),
            Type::RustBox(ptr) => check_type_box(cx, ptr),
            Type::UniquePtr(ptr) => check_type_unique_ptr(cx, ptr),
            Type::Ref(ty) => check_type_ref(cx, ty),
            Type::Slice(ty) => check_type_slice(cx, ty),
            _ => {}
        }
    }

    for api in cx.apis {
        match api {
            Api::Struct(strct) => check_api_struct(cx, strct),
            Api::CxxFunction(efn) | Api::RustFunction(efn) => check_api_fn(cx, efn),
            _ => {}
        }
    }

    for api in cx.apis {
        if let Api::CxxFunction(efn) = api {
            check_mut_return_restriction(cx, efn);
        }
        if let Api::CxxFunction(efn) | Api::RustFunction(efn) = api {
            check_multiple_arg_lifetimes(cx, efn);
        }
    }

    ident::check_all(cx.apis, cx.errors);
}

impl Check<'_> {
    fn error(&mut self, sp: impl ToTokens, msg: impl Display) {
        self.errors.push(Error::new_spanned(sp, msg));
    }
}

fn check_type_ident(cx: &mut Check, ident: &Ident) {
    if Atom::from(ident).is_none()
        && !cx.types.structs.contains_key(ident)
        && !cx.types.cxx.contains(ident)
        && !cx.types.rust.contains(ident)
    {
        cx.error(ident, "unsupported type");
    }
}

fn check_type_box(cx: &mut Check, ptr: &Ty1) {
    if let Type::Ident(ident) = &ptr.inner {
        if cx.types.cxx.contains(ident) {
            cx.error(ptr, error::BOX_CXX_TYPE.msg);
        }

        if Atom::from(ident).is_none() {
            return;
        }
    }

    cx.error(ptr, "unsupported target type of Box");
}

fn check_type_unique_ptr(cx: &mut Check, ptr: &Ty1) {
    if let Type::Ident(ident) = &ptr.inner {
        if cx.types.rust.contains(ident) {
            cx.error(ptr, "unique_ptr of a Rust type is not supported yet");
        }

        match Atom::from(ident) {
            None | Some(CxxString) => return,
            _ => {}
        }
    }

    cx.error(ptr, "unsupported unique_ptr target type");
}

fn check_type_ref(cx: &mut Check, ty: &Ref) {
    if ty.lifetime.is_some() {
        cx.error(ty, "references with explicit lifetimes are not supported");
    }

    match ty.inner {
        Type::Fn(_) | Type::Void(_) => {}
        _ => return,
    }

    cx.error(ty, "unsupported reference type");
}

fn check_type_slice(cx: &mut Check, ty: &Slice) {
    cx.error(ty, "only &[u8] is supported so far, not other slice types");
}

fn check_api_struct(cx: &mut Check, strct: &Struct) {
    if strct.fields.is_empty() {
        let span = span_for_struct_error(strct);
        cx.error(span, "structs without any fields are not supported");
    }

    for field in &strct.fields {
        if is_unsized(cx, &field.ty) {
            let desc = describe(cx, &field.ty);
            let msg = format!("using {} by value is not supported", desc);
            cx.error(field, msg);
        }
        if let Type::Fn(_) = field.ty {
            cx.error(
                field,
                "function pointers in a struct field are not implemented yet",
            );
        }
    }
}

fn check_api_fn(cx: &mut Check, efn: &ExternFn) {
    if let Some(receiver) = &efn.receiver {
        if receiver.lifetime.is_some() {
            let span = span_for_receiver_error(receiver);
            cx.error(span, "references with explicit lifetimes are not supported");
        }
    }

    for arg in &efn.args {
        if is_unsized(cx, &arg.ty) {
            let desc = describe(cx, &arg.ty);
            let msg = format!("passing {} by value is not supported", desc);
            cx.error(arg, msg);
        }
        if let Type::Fn(_) = arg.ty {
            if efn.lang == Lang::Rust {
                cx.error(
                    arg,
                    "passing a function pointer from C++ to Rust is not implemented yet",
                );
            }
        }
    }

    if let Some(ty) = &efn.ret {
        if is_unsized(cx, ty) {
            let desc = describe(cx, ty);
            let msg = format!("returning {} by value is not supported", desc);
            cx.error(ty, msg);
        }
        if let Type::Fn(_) = ty {
            cx.error(ty, "returning a function pointer is not implemented yet");
        }
    }
}

fn check_mut_return_restriction(cx: &mut Check, efn: &ExternFn) {
    match &efn.ret {
        Some(Type::Ref(ty)) if ty.mutability.is_some() => {}
        _ => return,
    }

    for arg in &efn.args {
        if let Type::Ref(ty) = &arg.ty {
            if ty.mutability.is_some() {
                return;
            }
        }
    }

    cx.error(
        efn,
        "&mut return type is not allowed unless there is a &mut argument",
    );
}

fn check_multiple_arg_lifetimes(cx: &mut Check, efn: &ExternFn) {
    match &efn.ret {
        Some(Type::Ref(_)) => {}
        _ => return,
    }

    let mut reference_args = 0;
    for arg in &efn.args {
        if let Type::Ref(_) = &arg.ty {
            reference_args += 1;
        }
    }

    if efn.receiver.is_some() {
        reference_args += 1;
    }

    if reference_args != 1 {
        cx.error(
            efn,
            "functions that return a reference must take exactly one input reference",
        );
    }
}

fn is_unsized(cx: &mut Check, ty: &Type) -> bool {
    let ident = match ty {
        Type::Ident(ident) => ident,
        Type::Slice(_) | Type::Void(_) => return true,
        _ => return false,
    };
    ident == CxxString || cx.types.cxx.contains(ident) || cx.types.rust.contains(ident)
}

fn span_for_struct_error(strct: &Struct) -> TokenStream {
    let struct_token = strct.struct_token;
    let mut brace_token = Group::new(Delimiter::Brace, TokenStream::new());
    brace_token.set_span(strct.brace_token.span);
    quote!(#struct_token #brace_token)
}

fn span_for_receiver_error(receiver: &Receiver) -> TokenStream {
    let ampersand = receiver.ampersand;
    let lifetime = &receiver.lifetime;
    let mutability = receiver.mutability;
    if receiver.shorthand {
        let var = receiver.var;
        quote!(#ampersand #lifetime #mutability #var)
    } else {
        let ty = &receiver.ty;
        quote!(#ampersand #lifetime #mutability #ty)
    }
}

fn combine_errors(errors: Vec<Error>) -> Result<()> {
    let mut iter = errors.into_iter();
    let mut all_errors = match iter.next() {
        Some(err) => err,
        None => return Ok(()),
    };
    for err in iter {
        all_errors.combine(err);
    }
    Err(all_errors)
}

fn describe(cx: &mut Check, ty: &Type) -> String {
    match ty {
        Type::Ident(ident) => {
            if cx.types.structs.contains_key(ident) {
                "struct".to_owned()
            } else if cx.types.cxx.contains(ident) {
                "C++ type".to_owned()
            } else if cx.types.rust.contains(ident) {
                "opaque Rust type".to_owned()
            } else if Atom::from(ident) == Some(CxxString) {
                "C++ string".to_owned()
            } else {
                ident.to_string()
            }
        }
        Type::RustBox(_) => "Box".to_owned(),
        Type::UniquePtr(_) => "unique_ptr".to_owned(),
        Type::Ref(_) => "reference".to_owned(),
        Type::Str(_) => "&str".to_owned(),
        Type::Slice(_) => "slice".to_owned(),
        Type::SliceRefU8(_) => "&[u8]".to_owned(),
        Type::Fn(_) => "function pointer".to_owned(),
        Type::Void(_) => "()".to_owned(),
    }
}
