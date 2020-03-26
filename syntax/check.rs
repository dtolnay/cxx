use crate::syntax::atom::Atom::{self, *};
use crate::syntax::{error, ident, Api, ExternFn, Ref, Struct, Ty1, Type, Types, Var};
use proc_macro2::{Delimiter, Group, Ident, TokenStream};
use quote::quote;
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
            Type::Fn(_) => cx.errors.push(unimplemented_fn_type(ty)),
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

fn check_type_ident(cx: &mut Check, ident: &Ident) {
    if Atom::from(ident).is_none()
        && !cx.types.structs.contains_key(ident)
        && !cx.types.cxx.contains(ident)
        && !cx.types.rust.contains(ident)
    {
        cx.errors.push(unsupported_type(ident));
    }
}

fn check_type_box(cx: &mut Check, ptr: &Ty1) {
    if let Type::Ident(ident) = &ptr.inner {
        if cx.types.cxx.contains(ident) {
            cx.errors.push(unsupported_cxx_type_in_box(ptr));
        }
        if Atom::from(ident).is_none() {
            return;
        }
    }
    cx.errors.push(unsupported_box_target(ptr));
}

fn check_type_unique_ptr(cx: &mut Check, ptr: &Ty1) {
    if let Type::Ident(ident) = &ptr.inner {
        if cx.types.rust.contains(ident) {
            cx.errors.push(unsupported_rust_type_in_unique_ptr(ptr));
        }
        match Atom::from(ident) {
            None | Some(CxxString) => return,
            _ => {}
        }
    }
    cx.errors.push(unsupported_unique_ptr_target(ptr));
}

fn check_type_ref(cx: &mut Check, ty: &Ref) {
    if let Type::Void(_) = ty.inner {
        cx.errors.push(unsupported_reference_type(ty));
    }
}

fn check_api_struct(cx: &mut Check, strct: &Struct) {
    if strct.fields.is_empty() {
        cx.errors.push(struct_empty(strct));
    }
    for field in &strct.fields {
        if is_unsized(cx, &field.ty) {
            cx.errors.push(field_by_value(field, cx.types));
        }
    }
}

fn check_api_fn(cx: &mut Check, efn: &ExternFn) {
    for arg in &efn.args {
        if is_unsized(cx, &arg.ty) {
            cx.errors.push(argument_by_value(arg, cx.types));
        }
    }
    if let Some(ty) = &efn.ret {
        if is_unsized(cx, ty) {
            cx.errors.push(return_by_value(ty, cx.types));
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

    cx.errors.push(Error::new_spanned(
        efn,
        "&mut return type is not allowed unless there is a &mut argument",
    ));
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

    if reference_args != 1 {
        cx.errors.push(Error::new_spanned(
            efn,
            "functions that return a reference must take exactly one input reference",
        ));
    }
}

fn is_unsized(cx: &mut Check, ty: &Type) -> bool {
    let ident = match ty {
        Type::Ident(ident) => ident,
        Type::Void(_) => return true,
        _ => return false,
    };
    ident == CxxString || cx.types.cxx.contains(ident) || cx.types.rust.contains(ident)
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

fn describe(ty: &Type, types: &Types) -> String {
    match ty {
        Type::Ident(ident) => {
            if types.structs.contains_key(ident) {
                "struct".to_owned()
            } else if types.cxx.contains(ident) {
                "C++ type".to_owned()
            } else if types.rust.contains(ident) {
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
        Type::Fn(_) => "function pointer".to_owned(),
        Type::Void(_) => "()".to_owned(),
    }
}

fn unsupported_type(ident: &Ident) -> Error {
    Error::new(ident.span(), "unsupported type")
}

fn unsupported_reference_type(ty: &Ref) -> Error {
    Error::new_spanned(ty, "unsupported reference type")
}

fn unsupported_cxx_type_in_box(unique_ptr: &Ty1) -> Error {
    Error::new_spanned(unique_ptr, error::BOX_CXX_TYPE.msg)
}

fn unsupported_box_target(unique_ptr: &Ty1) -> Error {
    Error::new_spanned(unique_ptr, "unsupported target type of Box")
}

fn unsupported_rust_type_in_unique_ptr(unique_ptr: &Ty1) -> Error {
    Error::new_spanned(unique_ptr, "unique_ptr of a Rust type is not supported yet")
}

fn unsupported_unique_ptr_target(unique_ptr: &Ty1) -> Error {
    Error::new_spanned(unique_ptr, "unsupported unique_ptr target type")
}

fn struct_empty(strct: &Struct) -> Error {
    let struct_token = strct.struct_token;
    let mut brace_token = Group::new(Delimiter::Brace, TokenStream::new());
    brace_token.set_span(strct.brace_token.span);
    let span = quote!(#struct_token #brace_token);
    Error::new_spanned(span, "structs without any fields are not supported")
}

fn field_by_value(field: &Var, types: &Types) -> Error {
    let desc = describe(&field.ty, types);
    let message = format!("using {} by value is not supported", desc);
    Error::new_spanned(field, message)
}

fn argument_by_value(arg: &Var, types: &Types) -> Error {
    let desc = describe(&arg.ty, types);
    let message = format!("passing {} by value is not supported", desc);
    Error::new_spanned(arg, message)
}

fn return_by_value(ty: &Type, types: &Types) -> Error {
    let desc = describe(ty, types);
    let message = format!("returning {} by value is not supported", desc);
    Error::new_spanned(ty, message)
}

fn unimplemented_fn_type(ty: &Type) -> Error {
    Error::new_spanned(ty, "function pointer support is not implemented yet")
}
