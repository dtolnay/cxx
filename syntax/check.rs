use crate::syntax::atom::Atom::{self, *};
use crate::syntax::report::Errors;
use crate::syntax::types::TrivialReason;
use crate::syntax::{
    error, ident, Api, Enum, ExternFn, ExternType, Impl, Lang, Receiver, Ref, Slice, Struct, Ty1,
    Type, Types,
};
use proc_macro2::{Delimiter, Group, Ident, TokenStream};
use quote::{quote, ToTokens};
use std::fmt::Display;

pub(crate) struct Check<'a> {
    apis: &'a [Api],
    types: &'a Types<'a>,
    errors: &'a mut Errors,
}

pub(crate) fn typecheck(cx: &mut Errors, apis: &[Api], types: &Types) {
    do_typecheck(&mut Check {
        apis,
        types,
        errors: cx,
    });
}

fn do_typecheck(cx: &mut Check) {
    ident::check_all(cx, cx.apis);

    for ty in cx.types {
        match ty {
            Type::Ident(ident) => check_type_ident(cx, &ident.rust),
            Type::RustBox(ptr) => check_type_box(cx, ptr),
            Type::RustVec(ty) => check_type_rust_vec(cx, ty),
            Type::UniquePtr(ptr) => check_type_unique_ptr(cx, ptr),
            Type::CxxVector(ptr) => check_type_cxx_vector(cx, ptr),
            Type::Ref(ty) => check_type_ref(cx, ty),
            Type::Slice(ty) => check_type_slice(cx, ty),
            _ => {}
        }
    }

    for api in cx.apis {
        match api {
            Api::Struct(strct) => check_api_struct(cx, strct),
            Api::Enum(enm) => check_api_enum(cx, enm),
            Api::CxxType(ety) | Api::RustType(ety) => check_api_type(cx, ety),
            Api::CxxFunction(efn) | Api::RustFunction(efn) => check_api_fn(cx, efn),
            Api::Impl(imp) => check_api_impl(cx, imp),
            _ => {}
        }
    }
}

impl Check<'_> {
    pub(crate) fn error(&mut self, sp: impl ToTokens, msg: impl Display) {
        self.errors.error(sp, msg);
    }
}

fn check_type_ident(cx: &mut Check, ident: &Ident) {
    if Atom::from(ident).is_none()
        && !cx.types.structs.contains_key(ident)
        && !cx.types.enums.contains_key(ident)
        && !cx.types.cxx.contains(ident)
        && !cx.types.rust.contains(ident)
    {
        let msg = format!("unsupported type: {}", ident);
        cx.error(ident, &msg);
    }
}

fn check_type_box(cx: &mut Check, ptr: &Ty1) {
    if let Type::Ident(ident) = &ptr.inner {
        if cx.types.cxx.contains(&ident.rust)
            && !cx.types.structs.contains_key(&ident.rust)
            && !cx.types.enums.contains_key(&ident.rust)
        {
            cx.error(ptr, error::BOX_CXX_TYPE.msg);
        }

        if Atom::from(&ident.rust).is_none() {
            return;
        }
    }

    cx.error(ptr, "unsupported target type of Box");
}

fn check_type_rust_vec(cx: &mut Check, ty: &Ty1) {
    if let Type::Ident(ident) = &ty.inner {
        if cx.types.cxx.contains(&ident.rust)
            && !cx.types.structs.contains_key(&ident.rust)
            && !cx.types.enums.contains_key(&ident.rust)
        {
            cx.error(ty, "Rust Vec containing C++ type is not supported yet");
            return;
        }

        match Atom::from(&ident.rust) {
            None | Some(U8) | Some(U16) | Some(U32) | Some(U64) | Some(Usize) | Some(I8)
            | Some(I16) | Some(I32) | Some(I64) | Some(Isize) | Some(F32) | Some(F64)
            | Some(RustString) => return,
            Some(Bool) => { /* todo */ }
            Some(CxxString) => {}
        }
    }

    cx.error(ty, "unsupported element type of Vec");
}

fn check_type_unique_ptr(cx: &mut Check, ptr: &Ty1) {
    if let Type::Ident(ident) = &ptr.inner {
        if cx.types.rust.contains(&ident.rust) {
            cx.error(ptr, "unique_ptr of a Rust type is not supported yet");
        }

        match Atom::from(&ident.rust) {
            None | Some(CxxString) => return,
            _ => {}
        }
    } else if let Type::CxxVector(_) = &ptr.inner {
        return;
    }

    cx.error(ptr, "unsupported unique_ptr target type");
}

fn check_type_cxx_vector(cx: &mut Check, ptr: &Ty1) {
    if let Type::Ident(ident) = &ptr.inner {
        if cx.types.rust.contains(&ident.rust) {
            cx.error(
                ptr,
                "C++ vector containing a Rust type is not supported yet",
            );
        }

        match Atom::from(&ident.rust) {
            None | Some(U8) | Some(U16) | Some(U32) | Some(U64) | Some(Usize) | Some(I8)
            | Some(I16) | Some(I32) | Some(I64) | Some(Isize) | Some(F32) | Some(F64)
            | Some(CxxString) => return,
            Some(Bool) | Some(RustString) => {}
        }
    }

    cx.error(ptr, "unsupported vector target type");
}

fn check_type_ref(cx: &mut Check, ty: &Ref) {
    if ty.lifetime.is_some() {
        cx.error(ty, "references with explicit lifetimes are not supported");
    }

    match ty.inner {
        Type::Fn(_) | Type::Void(_) => {}
        Type::Ref(_) => {
            cx.error(ty, "C++ does not allow references to references");
            return;
        }
        _ => return,
    }

    cx.error(ty, "unsupported reference type");
}

fn check_type_slice(cx: &mut Check, ty: &Slice) {
    cx.error(ty, "only &[u8] is supported so far, not other slice types");
}

fn check_api_struct(cx: &mut Check, strct: &Struct) {
    let name = &strct.name;
    check_reserved_name(cx, &name.rust);

    if strct.fields.is_empty() {
        let span = span_for_struct_error(strct);
        cx.error(span, "structs without any fields are not supported");
    }

    if cx.types.cxx.contains(&name.rust) {
        if let Some(ety) = cx.types.untrusted.get(&name.rust) {
            let msg = "extern shared struct must be declared in an `unsafe extern` block";
            cx.error(ety, msg);
        }
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

fn check_api_enum(cx: &mut Check, enm: &Enum) {
    check_reserved_name(cx, &enm.name.rust);

    if enm.variants.is_empty() {
        let span = span_for_enum_error(enm);
        cx.error(span, "enums without any variants are not supported");
    }
}

fn check_api_type(cx: &mut Check, ety: &ExternType) {
    check_reserved_name(cx, &ety.name.rust);

    if let Some(reason) = cx.types.required_trivial.get(&ety.name.rust) {
        let what = match reason {
            TrivialReason::StructField(strct) => format!("a field of `{}`", strct.name.rust),
            TrivialReason::FunctionArgument(efn) => format!("an argument of `{}`", efn.name.rust),
            TrivialReason::FunctionReturn(efn) => format!("a return value of `{}`", efn.name.rust),
        };
        let msg = format!(
            "needs a cxx::ExternType impl in order to be used as {}",
            what,
        );
        cx.error(ety, msg);
    }
}

fn check_api_fn(cx: &mut Check, efn: &ExternFn) {
    if let Some(receiver) = &efn.receiver {
        let ref span = span_for_receiver_error(receiver);

        if receiver.ty.is_self() {
            let mutability = match receiver.mutability {
                Some(_) => "mut ",
                None => "",
            };
            let msg = format!(
                "unnamed receiver type is only allowed if the surrounding \
                 extern block contains exactly one extern type; \
                 use `self: &{mutability}TheType`",
                mutability = mutability,
            );
            cx.error(span, msg);
        } else if !cx.types.structs.contains_key(&receiver.ty.rust)
            && !cx.types.cxx.contains(&receiver.ty.rust)
            && !cx.types.rust.contains(&receiver.ty.rust)
        {
            cx.error(span, "unrecognized receiver type");
        }

        if receiver.lifetime.is_some() {
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

    if efn.lang == Lang::Cxx {
        check_mut_return_restriction(cx, efn);
    }

    check_multiple_arg_lifetimes(cx, efn);
}

fn check_api_impl(cx: &mut Check, imp: &Impl) {
    if let Type::UniquePtr(ty) | Type::CxxVector(ty) = &imp.ty {
        if let Type::Ident(inner) = &ty.inner {
            if Atom::from(&inner.rust).is_none() {
                return;
            }
        }
    }

    cx.error(imp, "unsupported Self type of explicit impl");
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

fn check_reserved_name(cx: &mut Check, ident: &Ident) {
    if ident == "Box"
        || ident == "UniquePtr"
        || ident == "Vec"
        || ident == "CxxVector"
        || Atom::from(ident).is_some()
    {
        cx.error(ident, "reserved name");
    }
}

fn is_unsized(cx: &mut Check, ty: &Type) -> bool {
    let ident = match ty {
        Type::Ident(ident) => &ident.rust,
        Type::CxxVector(_) | Type::Slice(_) | Type::Void(_) => return true,
        _ => return false,
    };
    ident == CxxString
        || cx.types.cxx.contains(ident)
            && !cx.types.structs.contains_key(ident)
            && !cx.types.enums.contains_key(ident)
            && !(cx.types.aliases.contains_key(ident)
                && cx.types.required_trivial.contains_key(ident))
        || cx.types.rust.contains(ident)
}

fn span_for_struct_error(strct: &Struct) -> TokenStream {
    let struct_token = strct.struct_token;
    let mut brace_token = Group::new(Delimiter::Brace, TokenStream::new());
    brace_token.set_span(strct.brace_token.span);
    quote!(#struct_token #brace_token)
}

fn span_for_enum_error(enm: &Enum) -> TokenStream {
    let enum_token = enm.enum_token;
    let mut brace_token = Group::new(Delimiter::Brace, TokenStream::new());
    brace_token.set_span(enm.brace_token.span);
    quote!(#enum_token #brace_token)
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

fn describe(cx: &mut Check, ty: &Type) -> String {
    match ty {
        Type::Ident(ident) => {
            if cx.types.structs.contains_key(&ident.rust) {
                "struct".to_owned()
            } else if cx.types.enums.contains_key(&ident.rust) {
                "enum".to_owned()
            } else if cx.types.aliases.contains_key(&ident.rust) {
                "C++ type".to_owned()
            } else if cx.types.cxx.contains(&ident.rust) {
                "opaque C++ type".to_owned()
            } else if cx.types.rust.contains(&ident.rust) {
                "opaque Rust type".to_owned()
            } else if Atom::from(&ident.rust) == Some(CxxString) {
                "C++ string".to_owned()
            } else {
                ident.rust.to_string()
            }
        }
        Type::RustBox(_) => "Box".to_owned(),
        Type::RustVec(_) => "Vec".to_owned(),
        Type::UniquePtr(_) => "unique_ptr".to_owned(),
        Type::Ref(_) => "reference".to_owned(),
        Type::Str(_) => "&str".to_owned(),
        Type::CxxVector(_) => "C++ vector".to_owned(),
        Type::Slice(_) => "slice".to_owned(),
        Type::SliceRefU8(_) => "&[u8]".to_owned(),
        Type::Fn(_) => "function pointer".to_owned(),
        Type::Void(_) => "()".to_owned(),
    }
}
