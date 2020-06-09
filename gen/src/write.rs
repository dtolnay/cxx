use crate::gen::out::OutFile;
use crate::gen::write_type::{
    indirect_return, write_atom, write_rust_function_decl_impl, write_rust_function_shim_decl,
    write_rust_function_shim_impl, WriteOptionType, WriteType,
};
use crate::gen::{include, Opt};
use crate::syntax::atom::Atom::{self, *};
use crate::syntax::namespace::Namespace;
use crate::syntax::{mangle, Api, Enum, ExternFn, ExternType, Struct, Type, Types};
use proc_macro2::Ident;
use std::collections::HashMap;

pub(super) fn gen(
    namespace: &Namespace,
    apis: &[Api],
    types: &Types,
    opt: Opt,
    header: bool,
) -> OutFile {
    let mut out_file = OutFile::new(namespace.clone(), header);
    let out = &mut out_file;

    if header {
        writeln!(out, "#pragma once");
    }

    out.include.extend(opt.include);
    for api in apis {
        if let Api::Include(include) = api {
            out.include.insert(include);
        }
    }

    write_includes(out, types);
    write_include_cxxbridge(out, apis, types);

    out.next_section();
    for name in namespace {
        writeln!(out, "namespace {} {{", name);
    }

    out.next_section();
    for api in apis {
        match api {
            Api::Struct(strct) => write_struct_decl(out, &strct.ident),
            Api::CxxType(ety) => write_struct_using(out, &ety.ident),
            Api::RustType(ety) => write_struct_decl(out, &ety.ident),
            _ => {}
        }
    }

    let mut methods_for_type = HashMap::new();
    for api in apis {
        if let Api::RustFunction(efn) = api {
            if let Some(receiver) = &efn.sig.receiver {
                methods_for_type
                    .entry(&receiver.ty)
                    .or_insert_with(Vec::new)
                    .push(efn);
            }
        }
    }

    for api in apis {
        match api {
            Api::Struct(strct) => {
                out.next_section();
                write_struct(out, strct);
            }
            Api::Enum(enm) => {
                out.next_section();
                if types.cxx.contains(&enm.ident) {
                    check_enum(out, enm);
                } else {
                    write_enum(out, enm);
                }
            }
            Api::RustType(ety) => {
                if let Some(methods) = methods_for_type.get(&ety.ident) {
                    out.next_section();
                    write_struct_with_methods(out, ety, methods);
                }
            }
            _ => {}
        }
    }

    if !header {
        out.begin_block("extern \"C\"");
        write_exception_glue(out, apis);
        for api in apis {
            let (efn, write): (_, fn(_, _, _)) = match api {
                Api::CxxFunction(efn) => (efn, write_cxx_function_shim),
                Api::RustFunction(efn) => (efn, write_rust_function_decl),
                _ => continue,
            };
            out.next_section();
            write(out, efn, types);
        }
        out.end_block("extern \"C\"");
    }

    for api in apis {
        if let Api::RustFunction(efn) = api {
            out.next_section();
            write_rust_function_shim(out, efn, types);
        }
    }

    out.next_section();
    for name in namespace.iter().rev() {
        writeln!(out, "}} // namespace {}", name);
    }

    if !header {
        out.next_section();
        write_generic_instantiations(out, types);
    }

    out.prepend(out.include.to_string());

    out_file
}

fn write_includes(out: &mut OutFile, types: &Types) {
    for ty in types {
        match ty {
            Type::Ident(ident) => match Atom::from(ident) {
                Some(U8) | Some(U16) | Some(U32) | Some(U64) | Some(I8) | Some(I16) | Some(I32)
                | Some(I64) => out.include.cstdint = true,
                Some(Usize) => out.include.cstddef = true,
                Some(CxxString) => out.include.string = true,
                Some(Bool) | Some(Isize) | Some(F32) | Some(F64) | Some(RustString) | None => {}
            },
            Type::RustBox(_) => out.include.type_traits = true,
            Type::UniquePtr(_) => out.include.memory = true,
            Type::CxxVector(_) => out.include.vector = true,
            Type::SliceRefU8(_) => out.include.cstdint = true,
            _ => {}
        }
    }
}

fn write_include_cxxbridge(out: &mut OutFile, apis: &[Api], types: &Types) {
    let mut needs_rust_string = false;
    let mut needs_rust_str = false;
    let mut needs_rust_slice = false;
    let mut needs_rust_box = false;
    let mut needs_rust_vec = false;
    let mut needs_rust_fn = false;
    let mut needs_rust_isize = false;
    for ty in types {
        needs_rust_string = needs_rust_string || ty.needs_rust_string();
        needs_rust_str = needs_rust_str || ty.needs_rust_str();
        needs_rust_slice = needs_rust_slice || ty.needs_rust_slice();
        needs_rust_box = needs_rust_box || ty.needs_rust_box();
        needs_rust_vec = needs_rust_vec || ty.needs_rust_vec();
        needs_rust_fn = needs_rust_fn || ty.needs_rust_fn();
        needs_rust_isize = needs_rust_isize || ty.needs_rust_isize();
        out.include.type_traits = out.include.type_traits || ty.include_type_traits();
        out.include.array = out.include.array || ty.include_array();
        out.include.cstdint = out.include.cstdint || ty.include_cstdint();
        out.include.string = out.include.string || ty.include_string();
        out.include.base_tsd = out.include.base_tsd || ty.include_base_tsd();
    }

    let mut needs_rust_error = false;
    let mut needs_unsafe_bitcopy = false;
    let mut needs_manually_drop = false;
    let mut needs_maybe_uninit = false;
    let mut needs_trycatch = false;
    for api in apis {
        match api {
            Api::CxxFunction(efn) if !out.header => {
                if efn.throws {
                    needs_trycatch = true;
                }
                for arg in &efn.args {
                    let bitcopy = match arg.ty {
                        Type::RustVec(_) => true,
                        _ => arg.ty == RustString,
                    };
                    if bitcopy {
                        needs_unsafe_bitcopy = true;
                        break;
                    }
                }
            }
            Api::RustFunction(efn) if !out.header => {
                if efn.throws {
                    out.include.exception = true;
                    needs_rust_error = true;
                }
                for arg in &efn.args {
                    if arg.ty != RustString && types.needs_indirect_abi(&arg.ty) {
                        needs_manually_drop = true;
                        break;
                    }
                }
                if let Some(ret) = &efn.ret {
                    if types.needs_indirect_abi(ret) {
                        needs_maybe_uninit = true;
                    }
                }
            }
            _ => {}
        }
    }

    out.begin_block("namespace rust");
    out.begin_block("inline namespace cxxbridge03");

    if needs_rust_string
        || needs_rust_str
        || needs_rust_slice
        || needs_rust_box
        || needs_rust_vec
        || needs_rust_fn
        || needs_rust_error
        || needs_rust_isize
        || needs_unsafe_bitcopy
        || needs_manually_drop
        || needs_maybe_uninit
        || needs_trycatch
    {
        writeln!(out, "// #include \"rust/cxx.h\"");
    }

    if needs_rust_string || needs_rust_vec {
        out.next_section();
        writeln!(out, "struct unsafe_bitcopy_t;");
    }

    include::write(out, needs_rust_string, "CXXBRIDGE03_RUST_STRING");
    include::write(out, needs_rust_str, "CXXBRIDGE03_RUST_STR");
    include::write(out, needs_rust_slice, "CXXBRIDGE03_RUST_SLICE");
    include::write(out, needs_rust_box, "CXXBRIDGE03_RUST_BOX");
    include::write(out, needs_rust_vec, "CXXBRIDGE03_RUST_VEC");
    include::write(out, needs_rust_fn, "CXXBRIDGE03_RUST_FN");
    include::write(out, needs_rust_error, "CXXBRIDGE03_RUST_ERROR");
    include::write(out, needs_rust_isize, "CXXBRIDGE03_RUST_ISIZE");
    include::write(out, needs_unsafe_bitcopy, "CXXBRIDGE03_RUST_BITCOPY");

    if needs_manually_drop {
        out.next_section();
        out.include.utility = true;
        writeln!(out, "template <typename T>");
        writeln!(out, "union ManuallyDrop {{");
        writeln!(out, "  T value;");
        writeln!(
            out,
            "  ManuallyDrop(T &&value) : value(::std::move(value)) {{}}",
        );
        writeln!(out, "  ~ManuallyDrop() {{}}");
        writeln!(out, "}};");
    }

    if needs_maybe_uninit {
        out.next_section();
        writeln!(out, "template <typename T>");
        writeln!(out, "union MaybeUninit {{");
        writeln!(out, "  T value;");
        writeln!(out, "  MaybeUninit() {{}}");
        writeln!(out, "  ~MaybeUninit() {{}}");
        writeln!(out, "}};");
    }

    out.end_block("namespace cxxbridge03");

    if needs_trycatch {
        out.begin_block("namespace behavior");
        out.include.exception = true;
        out.include.type_traits = true;
        out.include.utility = true;
        writeln!(out, "class missing {{}};");
        writeln!(out, "missing trycatch(...);");
        writeln!(out);
        writeln!(out, "template <typename Try, typename Fail>");
        writeln!(out, "static typename std::enable_if<");
        writeln!(
            out,
            "    std::is_same<decltype(trycatch(std::declval<Try>(), std::declval<Fail>())),",
        );
        writeln!(out, "                 missing>::value>::type");
        writeln!(out, "trycatch(Try &&func, Fail &&fail) noexcept try {{");
        writeln!(out, "  func();");
        writeln!(out, "}} catch (const ::std::exception &e) {{");
        writeln!(out, "  fail(e.what());");
        writeln!(out, "}}");
        out.end_block("namespace behavior");
    }

    out.end_block("namespace rust");
}

fn write_struct(out: &mut OutFile, strct: &Struct) {
    for line in strct.doc.to_string().lines() {
        writeln!(out, "//{}", line);
    }
    writeln!(out, "struct {} final {{", strct.ident);
    for field in &strct.fields {
        write!(out, "  ");
        field.ty.write_type_space(out);
        writeln!(out, "{};", field.ident);
    }
    writeln!(out, "}};");
}

fn write_struct_decl(out: &mut OutFile, ident: &Ident) {
    writeln!(out, "struct {};", ident);
}

fn write_struct_using(out: &mut OutFile, ident: &Ident) {
    writeln!(out, "using {} = {};", ident, ident);
}

fn write_struct_with_methods(out: &mut OutFile, ety: &ExternType, methods: &[&ExternFn]) {
    for line in ety.doc.to_string().lines() {
        writeln!(out, "//{}", line);
    }
    writeln!(out, "struct {} final {{", ety.ident);
    writeln!(out, "  {}() = delete;", ety.ident);
    writeln!(out, "  {}(const {} &) = delete;", ety.ident, ety.ident);
    for method in methods {
        write!(out, "  ");
        let sig = &method.sig;
        let local_name = method.ident.to_string();
        write_rust_function_shim_decl(out, &local_name, sig, false);
        writeln!(out, ";");
    }
    writeln!(out, "}};");
}

fn write_enum(out: &mut OutFile, enm: &Enum) {
    for line in enm.doc.to_string().lines() {
        writeln!(out, "//{}", line);
    }
    write!(out, "enum class {} : ", enm.ident);
    write_atom(out, enm.repr);
    writeln!(out, " {{");
    for variant in &enm.variants {
        writeln!(out, "  {} = {},", variant.ident, variant.discriminant);
    }
    writeln!(out, "}};");
}

fn check_enum(out: &mut OutFile, enm: &Enum) {
    write!(out, "static_assert(sizeof({}) == sizeof(", enm.ident);
    write_atom(out, enm.repr);
    writeln!(out, "), \"incorrect size\");");
    for variant in &enm.variants {
        write!(out, "static_assert(static_cast<");
        write_atom(out, enm.repr);
        writeln!(
            out,
            ">({}::{}) == {}, \"disagrees with the value in #[cxx::bridge]\");",
            enm.ident, variant.ident, variant.discriminant,
        );
    }
}

fn write_exception_glue(out: &mut OutFile, apis: &[Api]) {
    let mut has_cxx_throws = false;
    for api in apis {
        if let Api::CxxFunction(efn) = api {
            if efn.throws {
                has_cxx_throws = true;
                break;
            }
        }
    }

    if has_cxx_throws {
        out.next_section();
        writeln!(
            out,
            "const char *cxxbridge03$exception(const char *, size_t);",
        );
    }
}

fn write_cxx_function_shim(out: &mut OutFile, efn: &ExternFn, types: &Types) {
    if efn.throws {
        write!(out, "::rust::Str::Repr ");
    } else {
        efn.ret.write_extern_return_type_space(out, types);
    }
    let mangled = mangle::extern_fn(&out.namespace, efn);
    write!(out, "{}(", mangled);
    if let Some(receiver) = &efn.receiver {
        if receiver.mutability.is_none() {
            write!(out, "const ");
        }
        write!(out, "{} &self", receiver.ty);
    }
    for (i, arg) in efn.args.iter().enumerate() {
        if i > 0 || efn.receiver.is_some() {
            write!(out, ", ");
        }
        arg.ty.write_extern_arg_cxx(out, types, &arg.ident);
    }
    let indirect_return = indirect_return(efn, types);
    if indirect_return {
        if !efn.args.is_empty() {
            write!(out, ", ");
        }
        efn.ret
            .as_ref()
            .unwrap()
            .write_indirect_return_type_space(out);
        write!(out, "*return$");
    }
    writeln!(out, ") noexcept {{");
    write!(out, "  ");
    efn.ret.write_return_type(out);
    match &efn.receiver {
        None => write!(out, "(*{}$)(", efn.ident),
        Some(receiver) => write!(out, "({}::*{}$)(", receiver.ty, efn.ident),
    }
    for (i, arg) in efn.args.iter().enumerate() {
        if i > 0 {
            write!(out, ", ");
        }
        arg.ty.write_type(out);
    }
    write!(out, ")");
    if let Some(receiver) = &efn.receiver {
        if receiver.mutability.is_none() {
            write!(out, " const");
        }
    }
    write!(out, " = ");
    match &efn.receiver {
        None => write!(out, "{}", efn.ident),
        Some(receiver) => write!(out, "&{}::{}", receiver.ty, efn.ident),
    }
    writeln!(out, ";");
    write!(out, "  ");
    if efn.throws {
        writeln!(out, "::rust::Str::Repr throw$;");
        writeln!(out, "  ::rust::behavior::trycatch(");
        writeln!(out, "      [&] {{");
        write!(out, "        ");
    }
    if indirect_return {
        write!(out, "new (return$) ");
        efn.ret.as_ref().unwrap().write_indirect_return_type(out);
        write!(out, "(");
    } else if efn.ret.is_some() {
        write!(out, "return ");
    }
    efn.ret.write_return_prefix(out, indirect_return);
    match &efn.receiver {
        None => write!(out, "{}$(", efn.ident),
        Some(_) => write!(out, "(self.*{}$)(", efn.ident),
    }
    for (i, arg) in efn.args.iter().enumerate() {
        if i > 0 {
            write!(out, ", ");
        }
        arg.ty.write_return_args(out, types, &arg.ident);
    }
    write!(out, ")");
    efn.ret.write_return_suffix(out, indirect_return);
    if indirect_return {
        write!(out, ")");
    }
    writeln!(out, ";");
    if efn.throws {
        out.include.cstring = true;
        writeln!(out, "        throw$.ptr = nullptr;");
        writeln!(out, "      }},");
        writeln!(out, "      [&](const char *catch$) noexcept {{");
        writeln!(out, "        throw$.len = ::std::strlen(catch$);");
        writeln!(
            out,
            "        throw$.ptr = cxxbridge03$exception(catch$, throw$.len);",
        );
        writeln!(out, "      }});");
        writeln!(out, "  return throw$;");
    }
    writeln!(out, "}}");
    for arg in &efn.args {
        arg.ty
            .write_function_pointer_trampoline(out, efn, &arg.ident, types);
    }
}

fn write_rust_function_decl(out: &mut OutFile, efn: &ExternFn, types: &Types) {
    let link_name = mangle::extern_fn(&out.namespace, efn);
    let indirect_call = false;
    write_rust_function_decl_impl(out, &link_name, efn, types, indirect_call);
}

fn write_rust_function_shim(out: &mut OutFile, efn: &ExternFn, types: &Types) {
    for line in efn.doc.to_string().lines() {
        writeln!(out, "//{}", line);
    }
    let local_name = match &efn.sig.receiver {
        None => efn.ident.to_string(),
        Some(receiver) => format!("{}::{}", receiver.ty, efn.ident),
    };
    let invoke = mangle::extern_fn(&out.namespace, efn);
    let indirect_call = false;
    write_rust_function_shim_impl(out, &local_name, efn, types, &invoke, indirect_call);
}

fn write_generic_instantiations(out: &mut OutFile, types: &Types) {
    out.begin_block("extern \"C\"");
    for ty in types {
        ty.write_extern(out, types);
    }
    out.end_block("extern \"C\"");

    out.begin_block("namespace rust");
    out.begin_block("inline namespace cxxbridge03");
    for ty in types {
        ty.write_impl(out);
    }
    out.end_block("namespace cxxbridge03");
    out.end_block("namespace rust");
}
