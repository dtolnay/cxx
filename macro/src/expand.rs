use crate::derive::DeriveAttribute;
use crate::syntax::atom::Atom::{self, *};
use crate::syntax::file::Module;
use crate::syntax::report::Errors;
use crate::syntax::symbol::Symbol;
use crate::syntax::{
    self, check, mangle, Api, Enum, ExternFn, ExternType, Impl, Pair, ResolvableName, Signature,
    Struct, Type, TypeAlias, Types,
};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use std::mem;
use syn::{parse_quote, Result, Token};

pub fn bridge(mut ffi: Module) -> Result<TokenStream> {
    let ref mut errors = Errors::new();
    let content = mem::take(&mut ffi.content);
    let trusted = ffi.unsafety.is_some();
    let namespace = &ffi.namespace;
    let ref apis = syntax::parse_items(errors, content, trusted, namespace);
    let ref types = Types::collect(errors, apis);
    errors.propagate()?;
    check::typecheck(errors, apis, types);
    errors.propagate()?;

    Ok(expand(ffi, apis, types))
}

fn expand(ffi: Module, apis: &[Api], types: &Types) -> TokenStream {
    let mut expanded = TokenStream::new();
    let mut hidden = TokenStream::new();

    for api in apis {
        if let Api::RustType(ety) = api {
            expanded.extend(expand_rust_type(ety));
            hidden.extend(expand_rust_type_assert_sized(ety));
        }
    }

    for api in apis {
        match api {
            Api::Include(_) | Api::RustType(_) | Api::Impl(_) => {}
            Api::Struct(strct) => expanded.extend(expand_struct(strct)),
            Api::Enum(enm) => expanded.extend(expand_enum(enm)),
            Api::CxxType(ety) => {
                let ident = &ety.name.rust;
                if !types.structs.contains_key(ident) && !types.enums.contains_key(ident) {
                    expanded.extend(expand_cxx_type(ety));
                }
            }
            Api::CxxFunction(efn) => {
                expanded.extend(expand_cxx_function_shim(efn, types));
            }
            Api::RustFunction(efn) => hidden.extend(expand_rust_function_shim(efn, types)),
            Api::TypeAlias(alias) => {
                expanded.extend(expand_type_alias(alias));
                hidden.extend(expand_type_alias_verify(alias, types));
            }
        }
    }

    for ty in types {
        let explicit_impl = types.explicit_impls.get(ty);
        if let Type::RustBox(ty) = ty {
            if let Type::Ident(ident) = &ty.inner {
                if Atom::from(&ident.rust).is_none() {
                    hidden.extend(expand_rust_box(ident, types));
                }
            }
        } else if let Type::RustVec(ty) = ty {
            if let Type::Ident(ident) = &ty.inner {
                if Atom::from(&ident.rust).is_none() {
                    hidden.extend(expand_rust_vec(ident, types));
                }
            }
        } else if let Type::UniquePtr(ptr) = ty {
            if let Type::Ident(ident) = &ptr.inner {
                if Atom::from(&ident.rust).is_none()
                    && (explicit_impl.is_some() || !types.aliases.contains_key(&ident.rust))
                {
                    expanded.extend(expand_unique_ptr(ident, types, explicit_impl));
                }
            }
        } else if let Type::CxxVector(ptr) = ty {
            if let Type::Ident(ident) = &ptr.inner {
                if Atom::from(&ident.rust).is_none()
                    && (explicit_impl.is_some() || !types.aliases.contains_key(&ident.rust))
                {
                    // Generate impl for CxxVector<T> if T is a struct or opaque
                    // C++ type. Impl for primitives is already provided by cxx
                    // crate.
                    expanded.extend(expand_cxx_vector(ident, explicit_impl, types));
                }
            }
        }
    }

    // Work around https://github.com/rust-lang/rust/issues/67851.
    if !hidden.is_empty() {
        expanded.extend(quote! {
            #[doc(hidden)]
            const _: () = {
                #hidden
            };
        });
    }

    let attrs = ffi
        .attrs
        .into_iter()
        .filter(|attr| attr.path.is_ident("doc"));
    let vis = &ffi.vis;
    let ident = &ffi.ident;

    quote! {
        #(#attrs)*
        #[deny(improper_ctypes)]
        #[allow(non_snake_case)]
        #vis mod #ident {
            #expanded
        }
    }
}

fn expand_struct(strct: &Struct) -> TokenStream {
    let ident = &strct.name.rust;
    let doc = &strct.doc;
    let derives = DeriveAttribute(&strct.derives);
    let type_id = type_id(&strct.name);
    let fields = strct.fields.iter().map(|field| {
        // This span on the pub makes "private type in public interface" errors
        // appear in the right place.
        let vis = Token![pub](field.ident.span());
        quote!(#vis #field)
    });

    quote! {
        #doc
        #derives
        #[repr(C)]
        pub struct #ident {
            #(#fields,)*
        }

        unsafe impl ::cxx::ExternType for #ident {
            type Id = #type_id;
            type Kind = ::cxx::kind::Trivial;
        }
    }
}

fn expand_enum(enm: &Enum) -> TokenStream {
    let ident = &enm.name.rust;
    let doc = &enm.doc;
    let repr = enm.repr;
    let type_id = type_id(&enm.name);
    let variants = enm.variants.iter().map(|variant| {
        let variant_ident = &variant.ident;
        let discriminant = &variant.discriminant;
        Some(quote! {
            pub const #variant_ident: Self = #ident { repr: #discriminant };
        })
    });

    quote! {
        #doc
        #[derive(Copy, Clone, PartialEq, Eq)]
        #[repr(transparent)]
        pub struct #ident {
            pub repr: #repr,
        }

        #[allow(non_upper_case_globals)]
        impl #ident {
            #(#variants)*
        }

        unsafe impl ::cxx::ExternType for #ident {
            type Id = #type_id;
            type Kind = ::cxx::kind::Trivial;
        }
    }
}

fn expand_cxx_type(ety: &ExternType) -> TokenStream {
    let ident = &ety.name.rust;
    let doc = &ety.doc;
    let type_id = type_id(&ety.name);

    quote! {
        #doc
        #[repr(C)]
        pub struct #ident {
            _private: ::cxx::private::Opaque,
        }

        unsafe impl ::cxx::ExternType for #ident {
            type Id = #type_id;
            type Kind = ::cxx::kind::Opaque;
        }
    }
}

fn expand_cxx_function_decl(efn: &ExternFn, types: &Types) -> TokenStream {
    let receiver = efn.receiver.iter().map(|receiver| {
        let receiver_type = receiver.ty();
        quote!(_: #receiver_type)
    });
    let args = efn.args.iter().map(|arg| {
        let ident = &arg.ident;
        let ty = expand_extern_type(&arg.ty, types, true);
        if arg.ty == RustString {
            quote!(#ident: *const #ty)
        } else if let Type::RustVec(_) = arg.ty {
            quote!(#ident: *const #ty)
        } else if let Type::Fn(_) = arg.ty {
            quote!(#ident: ::cxx::private::FatFunction)
        } else if types.needs_indirect_abi(&arg.ty) {
            quote!(#ident: *mut #ty)
        } else {
            quote!(#ident: #ty)
        }
    });
    let all_args = receiver.chain(args);
    let ret = if efn.throws {
        quote!(-> ::cxx::private::Result)
    } else {
        expand_extern_return_type(&efn.ret, types, true)
    };
    let mut outparam = None;
    if indirect_return(efn, types) {
        let ret = expand_extern_type(efn.ret.as_ref().unwrap(), types, true);
        outparam = Some(quote!(__return: *mut #ret));
    }
    let link_name = mangle::extern_fn(efn, types);
    let local_name = format_ident!("__{}", efn.name.rust);
    quote! {
        #[link_name = #link_name]
        fn #local_name(#(#all_args,)* #outparam) #ret;
    }
}

fn expand_cxx_function_shim(efn: &ExternFn, types: &Types) -> TokenStream {
    let doc = &efn.doc;
    let decl = expand_cxx_function_decl(efn, types);
    let receiver = efn.receiver.iter().map(|receiver| {
        let ampersand = receiver.ampersand;
        let mutability = receiver.mutability;
        let var = receiver.var;
        quote!(#ampersand #mutability #var)
    });
    let args = efn.args.iter().map(|arg| quote!(#arg));
    let all_args = receiver.chain(args);
    let ret = if efn.throws {
        let ok = match &efn.ret {
            Some(ret) => quote!(#ret),
            None => quote!(()),
        };
        quote!(-> ::std::result::Result<#ok, ::cxx::Exception>)
    } else {
        expand_return_type(&efn.ret)
    };
    let indirect_return = indirect_return(efn, types);
    let receiver_var = efn
        .receiver
        .iter()
        .map(|receiver| receiver.var.to_token_stream());
    let arg_vars = efn.args.iter().map(|arg| {
        let var = &arg.ident;
        match &arg.ty {
            Type::Ident(ident) if ident.rust == RustString => {
                quote!(#var.as_mut_ptr() as *const ::cxx::private::RustString)
            }
            Type::RustBox(_) => quote!(::std::boxed::Box::into_raw(#var)),
            Type::UniquePtr(_) => quote!(::cxx::UniquePtr::into_raw(#var)),
            Type::RustVec(_) => quote!(#var.as_mut_ptr() as *const ::cxx::private::RustVec<_>),
            Type::Ref(ty) => match &ty.inner {
                Type::Ident(ident) if ident.rust == RustString => match ty.mutability {
                    None => quote!(::cxx::private::RustString::from_ref(#var)),
                    Some(_) => quote!(::cxx::private::RustString::from_mut(#var)),
                },
                Type::RustVec(vec) if vec.inner == RustString => match ty.mutability {
                    None => quote!(::cxx::private::RustVec::from_ref_vec_string(#var)),
                    Some(_) => quote!(::cxx::private::RustVec::from_mut_vec_string(#var)),
                },
                Type::RustVec(_) => match ty.mutability {
                    None => quote!(::cxx::private::RustVec::from_ref(#var)),
                    Some(_) => quote!(::cxx::private::RustVec::from_mut(#var)),
                },
                inner if types.is_considered_improper_ctype(inner) => match ty.mutability {
                    None => quote!(#var as *const #inner as *const ::std::ffi::c_void),
                    Some(_) => quote!(#var as *mut #inner as *mut ::std::ffi::c_void),
                },
                _ => quote!(#var),
            },
            Type::Str(_) => quote!(::cxx::private::RustStr::from(#var)),
            Type::SliceRefU8(_) => quote!(::cxx::private::RustSliceU8::from(#var)),
            ty if types.needs_indirect_abi(ty) => quote!(#var.as_mut_ptr()),
            _ => quote!(#var),
        }
    });
    let vars = receiver_var.chain(arg_vars);
    let trampolines = efn
        .args
        .iter()
        .filter_map(|arg| {
            if let Type::Fn(f) = &arg.ty {
                let var = &arg.ident;
                Some(expand_function_pointer_trampoline(efn, var, f, types))
            } else {
                None
            }
        })
        .collect::<TokenStream>();
    let mut setup = efn
        .args
        .iter()
        .filter(|arg| types.needs_indirect_abi(&arg.ty))
        .map(|arg| {
            let var = &arg.ident;
            // These are arguments for which C++ has taken ownership of the data
            // behind the mut reference it received.
            quote! {
                let mut #var = ::std::mem::MaybeUninit::new(#var);
            }
        })
        .collect::<TokenStream>();
    let local_name = format_ident!("__{}", efn.name.rust);
    let call = if indirect_return {
        let ret = expand_extern_type(efn.ret.as_ref().unwrap(), types, true);
        setup.extend(quote! {
            let mut __return = ::std::mem::MaybeUninit::<#ret>::uninit();
        });
        setup.extend(if efn.throws {
            quote! {
                #local_name(#(#vars,)* __return.as_mut_ptr()).exception()?;
            }
        } else {
            quote! {
                #local_name(#(#vars,)* __return.as_mut_ptr());
            }
        });
        quote!(__return.assume_init())
    } else if efn.throws {
        quote! {
            #local_name(#(#vars),*).exception()
        }
    } else {
        quote! {
            #local_name(#(#vars),*)
        }
    };
    let mut expr;
    if efn.throws && efn.sig.ret.is_none() {
        expr = call;
    } else {
        expr = match &efn.ret {
            None => call,
            Some(ret) => match ret {
                Type::Ident(ident) if ident.rust == RustString => quote!(#call.into_string()),
                Type::RustBox(_) => quote!(::std::boxed::Box::from_raw(#call)),
                Type::RustVec(vec) => {
                    if vec.inner == RustString {
                        quote!(#call.into_vec_string())
                    } else {
                        quote!(#call.into_vec())
                    }
                }
                Type::UniquePtr(_) => quote!(::cxx::UniquePtr::from_raw(#call)),
                Type::Ref(ty) => match &ty.inner {
                    Type::Ident(ident) if ident.rust == RustString => match ty.mutability {
                        None => quote!(#call.as_string()),
                        Some(_) => quote!(#call.as_mut_string()),
                    },
                    Type::RustVec(vec) if vec.inner == RustString => match ty.mutability {
                        None => quote!(#call.as_vec_string()),
                        Some(_) => quote!(#call.as_mut_vec_string()),
                    },
                    Type::RustVec(_) => match ty.mutability {
                        None => quote!(#call.as_vec()),
                        Some(_) => quote!(#call.as_mut_vec()),
                    },
                    inner if types.is_considered_improper_ctype(inner) => {
                        let mutability = ty.mutability;
                        quote!(&#mutability *#call.cast())
                    }
                    _ => call,
                },
                Type::Str(_) => quote!(#call.as_str()),
                Type::SliceRefU8(_) => quote!(#call.as_slice()),
                _ => call,
            },
        };
        if efn.throws {
            expr = quote!(::std::result::Result::Ok(#expr));
        }
    };
    let mut dispatch = quote!(#setup #expr);
    let unsafety = &efn.sig.unsafety;
    if unsafety.is_none() {
        dispatch = quote!(unsafe { #dispatch });
    }
    let ident = &efn.name.rust;
    let function_shim = quote! {
        #doc
        pub #unsafety fn #ident(#(#all_args,)*) #ret {
            extern "C" {
                #decl
            }
            #trampolines
            #dispatch
        }
    };
    match &efn.receiver {
        None => function_shim,
        Some(receiver) => {
            let receiver_type = &receiver.ty;
            quote!(impl #receiver_type { #function_shim })
        }
    }
}

fn expand_function_pointer_trampoline(
    efn: &ExternFn,
    var: &Ident,
    sig: &Signature,
    types: &Types,
) -> TokenStream {
    let c_trampoline = mangle::c_trampoline(efn, var, types);
    let r_trampoline = mangle::r_trampoline(efn, var, types);
    let local_name = parse_quote!(__);
    let catch_unwind_label = format!("::{}::{}", efn.name.rust, var);
    let shim = expand_rust_function_shim_impl(
        sig,
        types,
        &r_trampoline,
        local_name,
        catch_unwind_label,
        None,
    );

    quote! {
        let #var = ::cxx::private::FatFunction {
            trampoline: {
                extern "C" {
                    #[link_name = #c_trampoline]
                    fn trampoline();
                }
                #shim
                trampoline as usize as *const ()
            },
            ptr: #var as usize as *const (),
        };
    }
}

fn expand_rust_type(ety: &ExternType) -> TokenStream {
    let ident = &ety.name.rust;
    quote! {
        use super::#ident;
    }
}

fn expand_rust_type_assert_sized(ety: &ExternType) -> TokenStream {
    // Rustc will render as follows if not sized:
    //
    //     type TheirType;
    //     -----^^^^^^^^^-
    //     |    |
    //     |    doesn't have a size known at compile-time
    //     required by this bound in `ffi::_::__AssertSized`

    let ident = &ety.name.rust;
    let begin_span = Token![::](ety.type_token.span);
    let sized = quote_spanned! {ety.semi_token.span=>
        #begin_span std::marker::Sized
    };
    quote_spanned! {ident.span()=>
        let _ = {
            fn __AssertSized<T: ?#sized + #sized>() {}
            __AssertSized::<#ident>
        };
    }
}

fn expand_rust_function_shim(efn: &ExternFn, types: &Types) -> TokenStream {
    let link_name = mangle::extern_fn(efn, types);
    let local_name = format_ident!("__{}", efn.name.rust);
    let catch_unwind_label = format!("::{}", efn.name.rust);
    let invoke = Some(&efn.name.rust);
    expand_rust_function_shim_impl(
        efn,
        types,
        &link_name,
        local_name,
        catch_unwind_label,
        invoke,
    )
}

fn expand_rust_function_shim_impl(
    sig: &Signature,
    types: &Types,
    link_name: &Symbol,
    local_name: Ident,
    catch_unwind_label: String,
    invoke: Option<&Ident>,
) -> TokenStream {
    let receiver_var = sig
        .receiver
        .as_ref()
        .map(|receiver| quote_spanned!(receiver.var.span=> __self));
    let receiver = sig.receiver.as_ref().map(|receiver| {
        let receiver_type = receiver.ty();
        quote!(#receiver_var: #receiver_type)
    });
    let args = sig.args.iter().map(|arg| {
        let ident = &arg.ident;
        let ty = expand_extern_type(&arg.ty, types, false);
        if types.needs_indirect_abi(&arg.ty) {
            quote!(#ident: *mut #ty)
        } else {
            quote!(#ident: #ty)
        }
    });
    let all_args = receiver.into_iter().chain(args);

    let arg_vars = sig.args.iter().map(|arg| {
        let ident = &arg.ident;
        match &arg.ty {
            Type::Ident(i) if i.rust == RustString => {
                quote!(::std::mem::take((*#ident).as_mut_string()))
            }
            Type::RustBox(_) => quote!(::std::boxed::Box::from_raw(#ident)),
            Type::RustVec(vec) => {
                if vec.inner == RustString {
                    quote!(::std::mem::take((*#ident).as_mut_vec_string()))
                } else {
                    quote!(::std::mem::take((*#ident).as_mut_vec()))
                }
            }
            Type::UniquePtr(_) => quote!(::cxx::UniquePtr::from_raw(#ident)),
            Type::Ref(ty) => match &ty.inner {
                Type::Ident(i) if i.rust == RustString => match ty.mutability {
                    None => quote!(#ident.as_string()),
                    Some(_) => quote!(#ident.as_mut_string()),
                },
                Type::RustVec(vec) if vec.inner == RustString => match ty.mutability {
                    None => quote!(#ident.as_vec_string()),
                    Some(_) => quote!(#ident.as_mut_vec_string()),
                },
                Type::RustVec(_) => match ty.mutability {
                    None => quote!(#ident.as_vec()),
                    Some(_) => quote!(#ident.as_mut_vec()),
                },
                _ => quote!(#ident),
            },
            Type::Str(_) => quote!(#ident.as_str()),
            Type::SliceRefU8(_) => quote!(#ident.as_slice()),
            ty if types.needs_indirect_abi(ty) => quote!(::std::ptr::read(#ident)),
            _ => quote!(#ident),
        }
    });
    let vars = receiver_var.into_iter().chain(arg_vars);

    let wrap_super = invoke.map(|invoke| expand_rust_function_shim_super(sig, &local_name, invoke));

    let mut call = match invoke {
        Some(_) => quote!(#local_name),
        None => quote!(::std::mem::transmute::<*const (), #sig>(__extern)),
    };
    call.extend(quote! { (#(#vars),*) });

    let conversion = sig.ret.as_ref().and_then(|ret| match ret {
        Type::Ident(ident) if ident.rust == RustString => {
            Some(quote!(::cxx::private::RustString::from))
        }
        Type::RustBox(_) => Some(quote!(::std::boxed::Box::into_raw)),
        Type::RustVec(vec) => {
            if vec.inner == RustString {
                Some(quote!(::cxx::private::RustVec::from_vec_string))
            } else {
                Some(quote!(::cxx::private::RustVec::from))
            }
        }
        Type::UniquePtr(_) => Some(quote!(::cxx::UniquePtr::into_raw)),
        Type::Ref(ty) => match &ty.inner {
            Type::Ident(ident) if ident.rust == RustString => match ty.mutability {
                None => Some(quote!(::cxx::private::RustString::from_ref)),
                Some(_) => Some(quote!(::cxx::private::RustString::from_mut)),
            },
            Type::RustVec(vec) if vec.inner == RustString => match ty.mutability {
                None => Some(quote!(::cxx::private::RustVec::from_ref_vec_string)),
                Some(_) => Some(quote!(::cxx::private::RustVec::from_mut_vec_string)),
            },
            Type::RustVec(_) => match ty.mutability {
                None => Some(quote!(::cxx::private::RustVec::from_ref)),
                Some(_) => Some(quote!(::cxx::private::RustVec::from_mut)),
            },
            _ => None,
        },
        Type::Str(_) => Some(quote!(::cxx::private::RustStr::from)),
        Type::SliceRefU8(_) => Some(quote!(::cxx::private::RustSliceU8::from)),
        _ => None,
    });

    let mut expr = match conversion {
        None => call,
        Some(conversion) if !sig.throws => quote!(#conversion(#call)),
        Some(conversion) => quote!(::std::result::Result::map(#call, #conversion)),
    };

    let mut outparam = None;
    let indirect_return = indirect_return(sig, types);
    if indirect_return {
        let ret = expand_extern_type(sig.ret.as_ref().unwrap(), types, false);
        outparam = Some(quote!(__return: *mut #ret,));
    }
    if sig.throws {
        let out = match sig.ret {
            Some(_) => quote!(__return),
            None => quote!(&mut ()),
        };
        expr = quote!(::cxx::private::r#try(#out, #expr));
    } else if indirect_return {
        expr = quote!(::std::ptr::write(__return, #expr));
    }

    expr = quote!(::cxx::private::catch_unwind(__fn, move || #expr));

    let ret = if sig.throws {
        quote!(-> ::cxx::private::Result)
    } else {
        expand_extern_return_type(&sig.ret, types, false)
    };

    let pointer = match invoke {
        None => Some(quote!(__extern: *const ())),
        Some(_) => None,
    };

    quote! {
        #[doc(hidden)]
        #[export_name = #link_name]
        unsafe extern "C" fn #local_name(#(#all_args,)* #outparam #pointer) #ret {
            let __fn = concat!(module_path!(), #catch_unwind_label);
            #wrap_super
            #expr
        }
    }
}

// A wrapper like `fn f(x: Arg) { super::f(x) }` just to ensure we have the
// accurate unsafety declaration and no problematic elided lifetimes.
fn expand_rust_function_shim_super(
    sig: &Signature,
    local_name: &Ident,
    invoke: &Ident,
) -> TokenStream {
    let unsafety = sig.unsafety;

    let receiver_var = sig
        .receiver
        .as_ref()
        .map(|receiver| Ident::new("__self", receiver.var.span));
    let receiver = sig.receiver.iter().map(|receiver| {
        let receiver_type = receiver.ty();
        quote!(#receiver_var: #receiver_type)
    });
    let args = sig.args.iter().map(|arg| quote!(#arg));
    let all_args = receiver.chain(args);

    let ret = if let Some((result, _langle, rangle)) = sig.throws_tokens {
        let ok = match &sig.ret {
            Some(ret) => quote!(#ret),
            None => quote!(()),
        };
        let impl_trait = quote_spanned!(result.span=> impl);
        let display = quote_spanned!(rangle.span=> ::std::fmt::Display);
        quote!(-> ::std::result::Result<#ok, #impl_trait #display>)
    } else {
        expand_return_type(&sig.ret)
    };

    let arg_vars = sig.args.iter().map(|arg| &arg.ident);
    let vars = receiver_var.iter().chain(arg_vars);

    let span = invoke.span();
    let call = match &sig.receiver {
        None => quote_spanned!(span=> super::#invoke),
        Some(receiver) => {
            let receiver_type = &receiver.ty;
            quote_spanned!(span=> #receiver_type::#invoke)
        }
    };

    quote_spanned! {span=>
        #unsafety fn #local_name(#(#all_args,)*) #ret {
            #call(#(#vars,)*)
        }
    }
}

fn expand_type_alias(alias: &TypeAlias) -> TokenStream {
    let doc = &alias.doc;
    let ident = &alias.name.rust;
    let ty = &alias.ty;
    quote! {
        #doc
        pub type #ident = #ty;
    }
}

fn expand_type_alias_verify(alias: &TypeAlias, types: &Types) -> TokenStream {
    let ident = &alias.name.rust;
    let type_id = type_id(&alias.name);
    let begin_span = alias.type_token.span;
    let end_span = alias.semi_token.span;
    let begin = quote_spanned!(begin_span=> ::cxx::private::verify_extern_type::<);
    let end = quote_spanned!(end_span=> >);

    let mut verify = quote! {
        const _: fn() = #begin #ident, #type_id #end;
    };

    if types.required_trivial.contains_key(&alias.name.rust) {
        let begin = quote_spanned!(begin_span=> ::cxx::private::verify_extern_kind::<);
        verify.extend(quote! {
            const _: fn() = #begin #ident, ::cxx::kind::Trivial #end;
        });
    }

    verify
}

fn type_id(name: &Pair) -> TokenStream {
    let path = name.to_fully_qualified();
    quote! {
        ::cxx::type_id!(#path)
    }
}

fn expand_rust_box(ident: &ResolvableName, types: &Types) -> TokenStream {
    let link_prefix = format!("cxxbridge05$box${}$", types.resolve(ident).to_symbol());
    let link_uninit = format!("{}uninit", link_prefix);
    let link_drop = format!("{}drop", link_prefix);

    let local_prefix = format_ident!("{}__box_", &ident.rust);
    let local_uninit = format_ident!("{}uninit", local_prefix);
    let local_drop = format_ident!("{}drop", local_prefix);

    let span = ident.span();
    quote_spanned! {span=>
        #[doc(hidden)]
        #[export_name = #link_uninit]
        unsafe extern "C" fn #local_uninit(
            this: *mut ::std::boxed::Box<::std::mem::MaybeUninit<#ident>>,
        ) {
            ::std::ptr::write(
                this,
                ::std::boxed::Box::new(::std::mem::MaybeUninit::uninit()),
            );
        }
        #[doc(hidden)]
        #[export_name = #link_drop]
        unsafe extern "C" fn #local_drop(this: *mut ::std::boxed::Box<#ident>) {
            ::std::ptr::drop_in_place(this);
        }
    }
}

fn expand_rust_vec(elem: &ResolvableName, types: &Types) -> TokenStream {
    let link_prefix = format!("cxxbridge05$rust_vec${}$", elem.to_symbol(types));
    let link_new = format!("{}new", link_prefix);
    let link_drop = format!("{}drop", link_prefix);
    let link_len = format!("{}len", link_prefix);
    let link_data = format!("{}data", link_prefix);
    let link_reserve_total = format!("{}reserve_total", link_prefix);
    let link_set_len = format!("{}set_len", link_prefix);
    let link_stride = format!("{}stride", link_prefix);

    let local_prefix = format_ident!("{}__vec_", elem.rust);
    let local_new = format_ident!("{}new", local_prefix);
    let local_drop = format_ident!("{}drop", local_prefix);
    let local_len = format_ident!("{}len", local_prefix);
    let local_data = format_ident!("{}data", local_prefix);
    let local_reserve_total = format_ident!("{}reserve_total", local_prefix);
    let local_set_len = format_ident!("{}set_len", local_prefix);
    let local_stride = format_ident!("{}stride", local_prefix);

    let span = elem.span();
    quote_spanned! {span=>
        #[doc(hidden)]
        #[export_name = #link_new]
        unsafe extern "C" fn #local_new(this: *mut ::cxx::private::RustVec<#elem>) {
            ::std::ptr::write(this, ::cxx::private::RustVec::new());
        }
        #[doc(hidden)]
        #[export_name = #link_drop]
        unsafe extern "C" fn #local_drop(this: *mut ::cxx::private::RustVec<#elem>) {
            ::std::ptr::drop_in_place(this);
        }
        #[doc(hidden)]
        #[export_name = #link_len]
        unsafe extern "C" fn #local_len(this: *const ::cxx::private::RustVec<#elem>) -> usize {
            (*this).len()
        }
        #[doc(hidden)]
        #[export_name = #link_data]
        unsafe extern "C" fn #local_data(this: *const ::cxx::private::RustVec<#elem>) -> *const #elem {
            (*this).as_ptr()
        }
        #[doc(hidden)]
        #[export_name = #link_reserve_total]
        unsafe extern "C" fn #local_reserve_total(this: *mut ::cxx::private::RustVec<#elem>, cap: usize) {
            (*this).reserve_total(cap);
        }
        #[doc(hidden)]
        #[export_name = #link_set_len]
        unsafe extern "C" fn #local_set_len(this: *mut ::cxx::private::RustVec<#elem>, len: usize) {
            (*this).set_len(len);
        }
        #[doc(hidden)]
        #[export_name = #link_stride]
        unsafe extern "C" fn #local_stride() -> usize {
            ::std::mem::size_of::<#elem>()
        }
    }
}

fn expand_unique_ptr(
    ident: &ResolvableName,
    types: &Types,
    explicit_impl: Option<&Impl>,
) -> TokenStream {
    let name = ident.rust.to_string();
    let prefix = format!("cxxbridge05$unique_ptr${}$", ident.to_symbol(types));
    let link_null = format!("{}null", prefix);
    let link_new = format!("{}new", prefix);
    let link_raw = format!("{}raw", prefix);
    let link_get = format!("{}get", prefix);
    let link_release = format!("{}release", prefix);
    let link_drop = format!("{}drop", prefix);

    let new_method =
        if types.structs.contains_key(&ident.rust) || types.aliases.contains_key(&ident.rust) {
            Some(quote! {
                fn __new(mut value: Self) -> *mut ::std::ffi::c_void {
                    extern "C" {
                        #[link_name = #link_new]
                        fn __new(this: *mut *mut ::std::ffi::c_void, value: *mut #ident);
                    }
                    let mut repr = ::std::ptr::null_mut::<::std::ffi::c_void>();
                    unsafe { __new(&mut repr, &mut value) }
                    repr
                }
            })
        } else {
            None
        };

    let begin_span =
        explicit_impl.map_or_else(Span::call_site, |explicit| explicit.impl_token.span);
    let end_span = explicit_impl.map_or_else(Span::call_site, |explicit| explicit.brace_token.span);
    let unsafe_token = format_ident!("unsafe", span = begin_span);

    quote_spanned! {end_span=>
        #unsafe_token impl ::cxx::private::UniquePtrTarget for #ident {
            const __NAME: &'static dyn ::std::fmt::Display = &#name;
            fn __null() -> *mut ::std::ffi::c_void {
                extern "C" {
                    #[link_name = #link_null]
                    fn __null(this: *mut *mut ::std::ffi::c_void);
                }
                let mut repr = ::std::ptr::null_mut::<::std::ffi::c_void>();
                unsafe { __null(&mut repr) }
                repr
            }
            #new_method
            unsafe fn __raw(raw: *mut Self) -> *mut ::std::ffi::c_void {
                extern "C" {
                    #[link_name = #link_raw]
                    fn __raw(this: *mut *mut ::std::ffi::c_void, raw: *mut #ident);
                }
                let mut repr = ::std::ptr::null_mut::<::std::ffi::c_void>();
                __raw(&mut repr, raw);
                repr
            }
            unsafe fn __get(repr: *mut ::std::ffi::c_void) -> *const Self {
                extern "C" {
                    #[link_name = #link_get]
                    fn __get(this: *const *mut ::std::ffi::c_void) -> *const #ident;
                }
                __get(&repr)
            }
            unsafe fn __release(mut repr: *mut ::std::ffi::c_void) -> *mut Self {
                extern "C" {
                    #[link_name = #link_release]
                    fn __release(this: *mut *mut ::std::ffi::c_void) -> *mut #ident;
                }
                __release(&mut repr)
            }
            unsafe fn __drop(mut repr: *mut ::std::ffi::c_void) {
                extern "C" {
                    #[link_name = #link_drop]
                    fn __drop(this: *mut *mut ::std::ffi::c_void);
                }
                __drop(&mut repr);
            }
        }
    }
}

fn expand_cxx_vector(
    elem: &ResolvableName,
    explicit_impl: Option<&Impl>,
    types: &Types,
) -> TokenStream {
    let _ = explicit_impl;
    let name = elem.rust.to_string();
    let prefix = format!("cxxbridge05$std$vector${}$", elem.to_symbol(types));
    let link_size = format!("{}size", prefix);
    let link_get_unchecked = format!("{}get_unchecked", prefix);
    let unique_ptr_prefix = format!(
        "cxxbridge05$unique_ptr$std$vector${}$",
        elem.to_symbol(types)
    );
    let link_unique_ptr_null = format!("{}null", unique_ptr_prefix);
    let link_unique_ptr_raw = format!("{}raw", unique_ptr_prefix);
    let link_unique_ptr_get = format!("{}get", unique_ptr_prefix);
    let link_unique_ptr_release = format!("{}release", unique_ptr_prefix);
    let link_unique_ptr_drop = format!("{}drop", unique_ptr_prefix);

    let begin_span =
        explicit_impl.map_or_else(Span::call_site, |explicit| explicit.impl_token.span);
    let end_span = explicit_impl.map_or_else(Span::call_site, |explicit| explicit.brace_token.span);
    let unsafe_token = format_ident!("unsafe", span = begin_span);

    quote_spanned! {end_span=>
        #unsafe_token impl ::cxx::private::VectorElement for #elem {
            const __NAME: &'static dyn ::std::fmt::Display = &#name;
            fn __vector_size(v: &::cxx::CxxVector<Self>) -> usize {
                extern "C" {
                    #[link_name = #link_size]
                    fn __vector_size(_: &::cxx::CxxVector<#elem>) -> usize;
                }
                unsafe { __vector_size(v) }
            }
            unsafe fn __get_unchecked(v: &::cxx::CxxVector<Self>, pos: usize) -> *const Self {
                extern "C" {
                    #[link_name = #link_get_unchecked]
                    fn __get_unchecked(_: &::cxx::CxxVector<#elem>, _: usize) -> *const #elem;
                }
                __get_unchecked(v, pos)
            }
            fn __unique_ptr_null() -> *mut ::std::ffi::c_void {
                extern "C" {
                    #[link_name = #link_unique_ptr_null]
                    fn __unique_ptr_null(this: *mut *mut ::std::ffi::c_void);
                }
                let mut repr = ::std::ptr::null_mut::<::std::ffi::c_void>();
                unsafe { __unique_ptr_null(&mut repr) }
                repr
            }
            unsafe fn __unique_ptr_raw(raw: *mut ::cxx::CxxVector<Self>) -> *mut ::std::ffi::c_void {
                extern "C" {
                    #[link_name = #link_unique_ptr_raw]
                    fn __unique_ptr_raw(this: *mut *mut ::std::ffi::c_void, raw: *mut ::cxx::CxxVector<#elem>);
                }
                let mut repr = ::std::ptr::null_mut::<::std::ffi::c_void>();
                __unique_ptr_raw(&mut repr, raw);
                repr
            }
            unsafe fn __unique_ptr_get(repr: *mut ::std::ffi::c_void) -> *const ::cxx::CxxVector<Self> {
                extern "C" {
                    #[link_name = #link_unique_ptr_get]
                    fn __unique_ptr_get(this: *const *mut ::std::ffi::c_void) -> *const ::cxx::CxxVector<#elem>;
                }
                __unique_ptr_get(&repr)
            }
            unsafe fn __unique_ptr_release(mut repr: *mut ::std::ffi::c_void) -> *mut ::cxx::CxxVector<Self> {
                extern "C" {
                    #[link_name = #link_unique_ptr_release]
                    fn __unique_ptr_release(this: *mut *mut ::std::ffi::c_void) -> *mut ::cxx::CxxVector<#elem>;
                }
                __unique_ptr_release(&mut repr)
            }
            unsafe fn __unique_ptr_drop(mut repr: *mut ::std::ffi::c_void) {
                extern "C" {
                    #[link_name = #link_unique_ptr_drop]
                    fn __unique_ptr_drop(this: *mut *mut ::std::ffi::c_void);
                }
                __unique_ptr_drop(&mut repr);
            }
        }
    }
}

fn expand_return_type(ret: &Option<Type>) -> TokenStream {
    match ret {
        Some(ret) => quote!(-> #ret),
        None => TokenStream::new(),
    }
}

fn indirect_return(sig: &Signature, types: &Types) -> bool {
    sig.ret
        .as_ref()
        .map_or(false, |ret| sig.throws || types.needs_indirect_abi(ret))
}

fn expand_extern_type(ty: &Type, types: &Types, proper: bool) -> TokenStream {
    match ty {
        Type::Ident(ident) if ident.rust == RustString => quote!(::cxx::private::RustString),
        Type::RustBox(ty) | Type::UniquePtr(ty) => {
            let inner = expand_extern_type(&ty.inner, types, proper);
            quote!(*mut #inner)
        }
        Type::RustVec(ty) => {
            let elem = expand_extern_type(&ty.inner, types, proper);
            quote!(::cxx::private::RustVec<#elem>)
        }
        Type::Ref(ty) => {
            let mutability = ty.mutability;
            match &ty.inner {
                Type::Ident(ident) if ident.rust == RustString => {
                    quote!(&#mutability ::cxx::private::RustString)
                }
                Type::RustVec(ty) => {
                    let inner = expand_extern_type(&ty.inner, types, proper);
                    quote!(&#mutability ::cxx::private::RustVec<#inner>)
                }
                inner if proper && types.is_considered_improper_ctype(inner) => match mutability {
                    None => quote!(*const ::std::ffi::c_void),
                    Some(_) => quote!(*#mutability ::std::ffi::c_void),
                },
                _ => quote!(#ty),
            }
        }
        Type::Str(_) => quote!(::cxx::private::RustStr),
        Type::SliceRefU8(_) => quote!(::cxx::private::RustSliceU8),
        _ => quote!(#ty),
    }
}

fn expand_extern_return_type(ret: &Option<Type>, types: &Types, proper: bool) -> TokenStream {
    let ret = match ret {
        Some(ret) if !types.needs_indirect_abi(ret) => ret,
        _ => return TokenStream::new(),
    };
    let ty = expand_extern_type(ret, types, proper);
    quote!(-> #ty)
}
