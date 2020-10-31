use crate::gen::namespace_organizer::NamespaceEntries;
use crate::gen::out::OutFile;
use crate::gen::{include, Opt};
use crate::syntax::atom::Atom::{self, *};
use crate::syntax::symbol::Symbol;
use crate::syntax::{
    mangle, Api, CppName, Enum, ExternFn, ExternType, ResolvableName, Signature, Struct, Type,
    Types, Var,
};
use proc_macro2::Ident;
use std::collections::HashMap;

pub(super) fn gen<'a>(apis: &[Api], types: &'a Types, opt: &Opt, header: bool) -> OutFile<'a> {
    let mut out_file = OutFile::new(header, types);
    let out = &mut out_file;

    if header {
        writeln!(out.front, "#pragma once");
    }

    out.include.extend(&opt.include);
    for api in apis {
        if let Api::Include(include) = api {
            out.include.insert(include);
        }
    }

    write_includes(out);
    write_include_cxxbridge(out, apis);

    out.next_section();

    let apis_by_namespace = NamespaceEntries::new(apis);

    gen_namespace_contents(&apis_by_namespace, types, opt, header, out);

    if !header {
        out.next_section();
        write_generic_instantiations(out);
    }

    write!(out.front, "{}", out.include);

    out_file
}

fn gen_namespace_contents(
    ns_entries: &NamespaceEntries,
    types: &Types,
    opt: &Opt,
    header: bool,
    out: &mut OutFile,
) {
    let apis = ns_entries.entries();

    out.next_section();
    for api in apis.iter() {
        match api {
            Api::Struct(strct) => write_struct_decl(out, &strct.ident.cxx.ident),
            Api::CxxType(ety) => write_struct_using(out, &ety.ident.cxx),
            Api::RustType(ety) => write_struct_decl(out, &ety.ident.cxx.ident),
            _ => {}
        }
    }

    let mut methods_for_type = HashMap::new();
    for api in apis.iter() {
        if let Api::RustFunction(efn) = api {
            if let Some(receiver) = &efn.sig.receiver {
                methods_for_type
                    .entry(&receiver.ty.rust)
                    .or_insert_with(Vec::new)
                    .push(efn);
            }
        }
    }

    for api in apis {
        match api {
            Api::Struct(strct) => {
                out.next_section();
                if !types.cxx.contains(&strct.ident.rust) {
                    write_struct(out, strct);
                }
            }
            Api::Enum(enm) => {
                out.next_section();
                if types.cxx.contains(&enm.ident.rust) {
                    check_enum(out, enm);
                } else {
                    write_enum(out, enm);
                }
            }
            Api::RustType(ety) => {
                if let Some(methods) = methods_for_type.get(&ety.ident.rust) {
                    out.next_section();
                    write_struct_with_methods(out, ety, methods);
                }
            }
            _ => {}
        }
    }

    out.next_section();
    for api in apis {
        if let Api::TypeAlias(ety) = api {
            if types.required_trivial.contains_key(&ety.ident.rust) {
                check_trivial_extern_type(out, &ety.ident.cxx)
            }
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
            write(out, efn, &opt.cxx_impl_annotations);
        }
        out.end_block("extern \"C\"");
    }

    for api in apis {
        if let Api::RustFunction(efn) = api {
            out.next_section();
            write_rust_function_shim(out, efn);
        }
    }

    out.next_section();

    for (child_ns, child_ns_entries) in ns_entries.children() {
        writeln!(out, "namespace {} {{", child_ns);
        gen_namespace_contents(&child_ns_entries, types, opt, header, out);
        writeln!(out, "}} // namespace {}", child_ns);
    }
}

fn write_includes(out: &mut OutFile) {
    for ty in out.types {
        match ty {
            Type::Ident(ident) => match Atom::from(&ident.rust) {
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

fn write_include_cxxbridge(out: &mut OutFile, apis: &[Api]) {
    let mut needs_panic = false;
    let mut needs_rust_string = false;
    let mut needs_rust_str = false;
    let mut needs_rust_slice = false;
    let mut needs_rust_box = false;
    let mut needs_rust_vec = false;
    let mut needs_rust_fn = false;
    let mut needs_rust_isize = false;
    let mut needs_unsafe_bitcopy = false;
    for ty in out.types {
        match ty {
            Type::RustBox(_) => {
                out.include.new = true;
                out.include.type_traits = true;
                needs_rust_box = true;
            }
            Type::RustVec(_) => {
                out.include.array = true;
                out.include.new = true;
                out.include.type_traits = true;
                needs_panic = true;
                needs_rust_vec = true;
                needs_unsafe_bitcopy = true;
            }
            Type::Str(_) => {
                out.include.cstdint = true;
                out.include.string = true;
                needs_rust_str = true;
            }
            Type::Fn(_) => {
                needs_rust_fn = true;
            }
            Type::Slice(_) | Type::SliceRefU8(_) => {
                needs_rust_slice = true;
            }
            ty if ty == Isize => {
                out.include.basetsd = true;
                needs_rust_isize = true;
            }
            ty if ty == RustString => {
                out.include.array = true;
                out.include.cstdint = true;
                out.include.string = true;
                needs_rust_string = true;
            }
            _ => {}
        }
    }

    let mut needs_rust_error = false;
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
                    out.include.string = true;
                    needs_rust_str = true;
                    needs_rust_error = true;
                    needs_maybe_uninit = true;
                }
                for arg in &efn.args {
                    if arg.ty != RustString && out.types.needs_indirect_abi(&arg.ty) {
                        needs_manually_drop = true;
                        break;
                    }
                }
                if let Some(ret) = &efn.ret {
                    if out.types.needs_indirect_abi(ret) {
                        needs_maybe_uninit = true;
                    }
                }
            }
            _ => {}
        }
    }

    out.begin_block("namespace rust");
    out.begin_block("inline namespace cxxbridge05");

    if needs_panic
        || needs_rust_string
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
    {
        writeln!(out, "// #include \"rust/cxx.h\"");
    }

    include::write(out, needs_panic, "CXXBRIDGE05_PANIC");

    if needs_rust_string {
        out.next_section();
        writeln!(out, "struct unsafe_bitcopy_t;");
    }

    include::write(out, needs_rust_string, "CXXBRIDGE05_RUST_STRING");
    include::write(out, needs_rust_str, "CXXBRIDGE05_RUST_STR");
    include::write(out, needs_rust_slice, "CXXBRIDGE05_RUST_SLICE");
    include::write(out, needs_rust_box, "CXXBRIDGE05_RUST_BOX");
    include::write(out, needs_unsafe_bitcopy, "CXXBRIDGE05_RUST_BITCOPY");
    include::write(out, needs_rust_vec, "CXXBRIDGE05_RUST_VEC");
    include::write(out, needs_rust_fn, "CXXBRIDGE05_RUST_FN");
    include::write(out, needs_rust_error, "CXXBRIDGE05_RUST_ERROR");
    include::write(out, needs_rust_isize, "CXXBRIDGE05_RUST_ISIZE");

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

    out.end_block("namespace cxxbridge05");

    if needs_trycatch {
        out.begin_block("namespace behavior");
        out.include.exception = true;
        out.include.type_traits = true;
        out.include.utility = true;
        writeln!(out, "class missing {{}};");
        writeln!(out, "missing trycatch(...);");
        writeln!(out);
        writeln!(out, "template <typename Try, typename Fail>");
        writeln!(out, "static typename ::std::enable_if<");
        writeln!(
            out,
            "    ::std::is_same<decltype(trycatch(::std::declval<Try>(), ::std::declval<Fail>())),",
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
    let guard = format!("CXXBRIDGE05_STRUCT_{}", strct.ident.cxx.to_symbol());
    writeln!(out, "#ifndef {}", guard);
    writeln!(out, "#define {}", guard);
    for line in strct.doc.to_string().lines() {
        writeln!(out, "//{}", line);
    }
    writeln!(out, "struct {} final {{", strct.ident.cxx.ident);
    for field in &strct.fields {
        write!(out, "  ");
        write_type_space(out, &field.ty);
        writeln!(out, "{};", field.ident);
    }
    writeln!(out, "}};");
    writeln!(out, "#endif // {}", guard);
}

fn write_struct_decl(out: &mut OutFile, ident: &Ident) {
    writeln!(out, "struct {};", ident);
}

fn write_struct_using(out: &mut OutFile, ident: &CppName) {
    writeln!(
        out,
        "using {} = {};",
        ident.ident,
        ident.to_fully_qualified()
    );
}

fn write_struct_with_methods(out: &mut OutFile, ety: &ExternType, methods: &[&ExternFn]) {
    let guard = format!("CXXBRIDGE05_STRUCT_{}", ety.ident.cxx.to_symbol());
    writeln!(out, "#ifndef {}", guard);
    writeln!(out, "#define {}", guard);
    for line in ety.doc.to_string().lines() {
        writeln!(out, "//{}", line);
    }
    writeln!(out, "struct {} final {{", ety.ident.cxx.ident);
    writeln!(out, "  {}() = delete;", ety.ident.cxx.ident);
    writeln!(
        out,
        "  {}(const {} &) = delete;",
        ety.ident.cxx.ident, ety.ident.cxx.ident
    );
    for method in methods {
        write!(out, "  ");
        let sig = &method.sig;
        let local_name = method.ident.cxx.ident.to_string();
        write_rust_function_shim_decl(out, &local_name, sig, false);
        writeln!(out, ";");
    }
    writeln!(out, "}};");
    writeln!(out, "#endif // {}", guard);
}

fn write_enum(out: &mut OutFile, enm: &Enum) {
    let guard = format!("CXXBRIDGE05_ENUM_{}", enm.ident.cxx.to_symbol());
    writeln!(out, "#ifndef {}", guard);
    writeln!(out, "#define {}", guard);
    for line in enm.doc.to_string().lines() {
        writeln!(out, "//{}", line);
    }
    write!(out, "enum class {} : ", enm.ident.cxx.ident);
    write_atom(out, enm.repr);
    writeln!(out, " {{");
    for variant in &enm.variants {
        writeln!(out, "  {} = {},", variant.ident, variant.discriminant);
    }
    writeln!(out, "}};");
    writeln!(out, "#endif // {}", guard);
}

fn check_enum(out: &mut OutFile, enm: &Enum) {
    write!(
        out,
        "static_assert(sizeof({}) == sizeof(",
        enm.ident.cxx.ident
    );
    write_atom(out, enm.repr);
    writeln!(out, "), \"incorrect size\");");
    for variant in &enm.variants {
        write!(out, "static_assert(static_cast<");
        write_atom(out, enm.repr);
        writeln!(
            out,
            ">({}::{}) == {}, \"disagrees with the value in #[cxx::bridge]\");",
            enm.ident.cxx.ident, variant.ident, variant.discriminant,
        );
    }
}

fn check_trivial_extern_type(out: &mut OutFile, id: &CppName) {
    // NOTE: The following two static assertions are just nice-to-have and not
    // necessary for soundness. That's because triviality is always declared by
    // the user in the form of an unsafe impl of cxx::ExternType:
    //
    //     unsafe impl ExternType for MyType {
    //         type Id = cxx::type_id!("...");
    //         type Kind = cxx::kind::Trivial;
    //     }
    //
    // Since the user went on the record with their unsafe impl to unsafely
    // claim they KNOW that the type is trivial, it's fine for that to be on
    // them if that were wrong.
    //
    // There may be a legitimate reason we'll want to remove these assertions
    // for support of types that the programmer knows are Rust-movable despite
    // not being recognized as such by the C++ type system due to a move
    // constructor or destructor.

    let id = &id.to_fully_qualified();
    out.include.type_traits = true;
    writeln!(out, "static_assert(");
    writeln!(
        out,
        "    ::std::is_trivially_move_constructible<{}>::value,",
        id,
    );
    writeln!(
        out,
        "    \"type {} marked as Trivial in Rust is not trivially move constructible in C++\");",
        id,
    );
    writeln!(out, "static_assert(");
    writeln!(out, "    ::std::is_trivially_destructible<{}>::value,", id);
    writeln!(
        out,
        "    \"type {} marked as Trivial in Rust is not trivially destructible in C++\");",
        id,
    );
}

fn write_exception_glue(out: &mut OutFile, apis: &[&Api]) {
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
            "const char *cxxbridge05$exception(const char *, size_t);",
        );
    }
}

fn write_cxx_function_shim(out: &mut OutFile, efn: &ExternFn, impl_annotations: &Option<String>) {
    if !out.header {
        if let Some(annotation) = impl_annotations {
            write!(out, "{} ", annotation);
        }
    }
    if efn.throws {
        write!(out, "::rust::Str::Repr ");
    } else {
        write_extern_return_type_space(out, &efn.ret);
    }
    let mangled = mangle::extern_fn(efn, out.types);
    write!(out, "{}(", mangled);
    if let Some(receiver) = &efn.receiver {
        if receiver.mutability.is_none() {
            write!(out, "const ");
        }
        write!(
            out,
            "{} &self",
            out.types.resolve(&receiver.ty).to_fully_qualified()
        );
    }
    for (i, arg) in efn.args.iter().enumerate() {
        if i > 0 || efn.receiver.is_some() {
            write!(out, ", ");
        }
        if arg.ty == RustString {
            write!(out, "const ");
        } else if let Type::RustVec(_) = arg.ty {
            write!(out, "const ");
        }
        write_extern_arg(out, arg);
    }
    let indirect_return = indirect_return(efn, out.types);
    if indirect_return {
        if !efn.args.is_empty() || efn.receiver.is_some() {
            write!(out, ", ");
        }
        write_indirect_return_type_space(out, efn.ret.as_ref().unwrap());
        write!(out, "*return$");
    }
    writeln!(out, ") noexcept {{");
    write!(out, "  ");
    write_return_type(out, &efn.ret);
    match &efn.receiver {
        None => write!(out, "(*{}$)(", efn.ident.rust),
        Some(receiver) => write!(
            out,
            "({}::*{}$)(",
            out.types.resolve(&receiver.ty).to_fully_qualified(),
            efn.ident.rust
        ),
    }
    for (i, arg) in efn.args.iter().enumerate() {
        if i > 0 {
            write!(out, ", ");
        }
        write_type(out, &arg.ty);
    }
    write!(out, ")");
    if let Some(receiver) = &efn.receiver {
        if receiver.mutability.is_none() {
            write!(out, " const");
        }
    }
    write!(out, " = ");
    match &efn.receiver {
        None => write!(out, "{}", efn.ident.cxx.to_fully_qualified()),
        Some(receiver) => write!(
            out,
            "&{}::{}",
            out.types.resolve(&receiver.ty).to_fully_qualified(),
            efn.ident.cxx.ident
        ),
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
        out.include.new = true;
        write!(out, "new (return$) ");
        write_indirect_return_type(out, efn.ret.as_ref().unwrap());
        write!(out, "(");
    } else if efn.ret.is_some() {
        write!(out, "return ");
    }
    match &efn.ret {
        Some(Type::Ref(_)) => write!(out, "&"),
        Some(Type::Str(_)) if !indirect_return => write!(out, "::rust::Str::Repr("),
        Some(Type::SliceRefU8(_)) if !indirect_return => {
            write!(out, "::rust::Slice<uint8_t>::Repr(")
        }
        _ => {}
    }
    match &efn.receiver {
        None => write!(out, "{}$(", efn.ident.rust),
        Some(_) => write!(out, "(self.*{}$)(", efn.ident.rust),
    }
    for (i, arg) in efn.args.iter().enumerate() {
        if i > 0 {
            write!(out, ", ");
        }
        if let Type::RustBox(_) = &arg.ty {
            write_type(out, &arg.ty);
            write!(out, "::from_raw({})", arg.ident);
        } else if let Type::UniquePtr(_) = &arg.ty {
            write_type(out, &arg.ty);
            write!(out, "({})", arg.ident);
        } else if arg.ty == RustString {
            write!(
                out,
                "::rust::String(::rust::unsafe_bitcopy, *{})",
                arg.ident,
            );
        } else if let Type::RustVec(_) = arg.ty {
            write_type(out, &arg.ty);
            write!(out, "(::rust::unsafe_bitcopy, *{})", arg.ident);
        } else if out.types.needs_indirect_abi(&arg.ty) {
            out.include.utility = true;
            write!(out, "::std::move(*{})", arg.ident);
        } else {
            write!(out, "{}", arg.ident);
        }
    }
    write!(out, ")");
    match &efn.ret {
        Some(Type::RustBox(_)) => write!(out, ".into_raw()"),
        Some(Type::UniquePtr(_)) => write!(out, ".release()"),
        Some(Type::Str(_)) | Some(Type::SliceRefU8(_)) if !indirect_return => write!(out, ")"),
        _ => {}
    }
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
            "        throw$.ptr = cxxbridge05$exception(catch$, throw$.len);",
        );
        writeln!(out, "      }});");
        writeln!(out, "  return throw$;");
    }
    writeln!(out, "}}");
    for arg in &efn.args {
        if let Type::Fn(f) = &arg.ty {
            let var = &arg.ident;
            write_function_pointer_trampoline(out, efn, var, f);
        }
    }
}

fn write_function_pointer_trampoline(
    out: &mut OutFile,
    efn: &ExternFn,
    var: &Ident,
    f: &Signature,
) {
    out.next_section();
    let r_trampoline = mangle::r_trampoline(efn, var, out.types);
    let indirect_call = true;
    write_rust_function_decl_impl(out, &r_trampoline, f, indirect_call);

    out.next_section();
    let c_trampoline = mangle::c_trampoline(efn, var, out.types).to_string();
    write_rust_function_shim_impl(out, &c_trampoline, f, &r_trampoline, indirect_call);
}

fn write_rust_function_decl(out: &mut OutFile, efn: &ExternFn, _: &Option<String>) {
    let link_name = mangle::extern_fn(efn, out.types);
    let indirect_call = false;
    write_rust_function_decl_impl(out, &link_name, efn, indirect_call);
}

fn write_rust_function_decl_impl(
    out: &mut OutFile,
    link_name: &Symbol,
    sig: &Signature,
    indirect_call: bool,
) {
    if sig.throws {
        write!(out, "::rust::Str::Repr ");
    } else {
        write_extern_return_type_space(out, &sig.ret);
    }
    write!(out, "{}(", link_name);
    let mut needs_comma = false;
    if let Some(receiver) = &sig.receiver {
        if receiver.mutability.is_none() {
            write!(out, "const ");
        }
        write!(
            out,
            "{} &self",
            out.types.resolve(&receiver.ty).to_fully_qualified()
        );
        needs_comma = true;
    }
    for arg in &sig.args {
        if needs_comma {
            write!(out, ", ");
        }
        write_extern_arg(out, arg);
        needs_comma = true;
    }
    if indirect_return(sig, out.types) {
        if needs_comma {
            write!(out, ", ");
        }
        write_return_type(out, &sig.ret);
        write!(out, "*return$");
        needs_comma = true;
    }
    if indirect_call {
        if needs_comma {
            write!(out, ", ");
        }
        write!(out, "void *");
    }
    writeln!(out, ") noexcept;");
}

fn write_rust_function_shim(out: &mut OutFile, efn: &ExternFn) {
    for line in efn.doc.to_string().lines() {
        writeln!(out, "//{}", line);
    }
    let local_name = match &efn.sig.receiver {
        None => efn.ident.cxx.ident.to_string(),
        Some(receiver) => format!(
            "{}::{}",
            out.types.resolve(&receiver.ty).ident,
            efn.ident.cxx.ident
        ),
    };
    let invoke = mangle::extern_fn(efn, out.types);
    let indirect_call = false;
    write_rust_function_shim_impl(out, &local_name, efn, &invoke, indirect_call);
}

fn write_rust_function_shim_decl(
    out: &mut OutFile,
    local_name: &str,
    sig: &Signature,
    indirect_call: bool,
) {
    write_return_type(out, &sig.ret);
    write!(out, "{}(", local_name);
    for (i, arg) in sig.args.iter().enumerate() {
        if i > 0 {
            write!(out, ", ");
        }
        write_type_space(out, &arg.ty);
        write!(out, "{}", arg.ident);
    }
    if indirect_call {
        if !sig.args.is_empty() {
            write!(out, ", ");
        }
        write!(out, "void *extern$");
    }
    write!(out, ")");
    if let Some(receiver) = &sig.receiver {
        if receiver.mutability.is_none() {
            write!(out, " const");
        }
    }
    if !sig.throws {
        write!(out, " noexcept");
    }
}

fn write_rust_function_shim_impl(
    out: &mut OutFile,
    local_name: &str,
    sig: &Signature,
    invoke: &Symbol,
    indirect_call: bool,
) {
    if out.header && sig.receiver.is_some() {
        // We've already defined this inside the struct.
        return;
    }
    write_rust_function_shim_decl(out, local_name, sig, indirect_call);
    if out.header {
        writeln!(out, ";");
        return;
    }
    writeln!(out, " {{");
    for arg in &sig.args {
        if arg.ty != RustString && out.types.needs_indirect_abi(&arg.ty) {
            out.include.utility = true;
            write!(out, "  ::rust::ManuallyDrop<");
            write_type(out, &arg.ty);
            writeln!(out, "> {}$(::std::move({0}));", arg.ident);
        }
    }
    write!(out, "  ");
    let indirect_return = indirect_return(sig, out.types);
    if indirect_return {
        write!(out, "::rust::MaybeUninit<");
        write_type(out, sig.ret.as_ref().unwrap());
        writeln!(out, "> return$;");
        write!(out, "  ");
    } else if let Some(ret) = &sig.ret {
        write!(out, "return ");
        match ret {
            Type::RustBox(_) => {
                write_type(out, ret);
                write!(out, "::from_raw(");
            }
            Type::UniquePtr(_) => {
                write_type(out, ret);
                write!(out, "(");
            }
            Type::Ref(_) => write!(out, "*"),
            _ => {}
        }
    }
    if sig.throws {
        write!(out, "::rust::Str::Repr error$ = ");
    }
    write!(out, "{}(", invoke);
    if sig.receiver.is_some() {
        write!(out, "*this");
    }
    for (i, arg) in sig.args.iter().enumerate() {
        if i > 0 || sig.receiver.is_some() {
            write!(out, ", ");
        }
        match &arg.ty {
            Type::Str(_) => write!(out, "::rust::Str::Repr("),
            Type::SliceRefU8(_) => write!(out, "::rust::Slice<uint8_t>::Repr("),
            ty if out.types.needs_indirect_abi(ty) => write!(out, "&"),
            _ => {}
        }
        write!(out, "{}", arg.ident);
        match &arg.ty {
            Type::RustBox(_) => write!(out, ".into_raw()"),
            Type::UniquePtr(_) => write!(out, ".release()"),
            Type::Str(_) | Type::SliceRefU8(_) => write!(out, ")"),
            ty if ty != RustString && out.types.needs_indirect_abi(ty) => write!(out, "$.value"),
            _ => {}
        }
    }
    if indirect_return {
        if !sig.args.is_empty() {
            write!(out, ", ");
        }
        write!(out, "&return$.value");
    }
    if indirect_call {
        if !sig.args.is_empty() || indirect_return {
            write!(out, ", ");
        }
        write!(out, "extern$");
    }
    write!(out, ")");
    if !indirect_return {
        if let Some(ret) = &sig.ret {
            if let Type::RustBox(_) | Type::UniquePtr(_) = ret {
                write!(out, ")");
            }
        }
    }
    writeln!(out, ";");
    if sig.throws {
        writeln!(out, "  if (error$.ptr) {{");
        writeln!(out, "    throw ::rust::Error(error$);");
        writeln!(out, "  }}");
    }
    if indirect_return {
        out.include.utility = true;
        writeln!(out, "  return ::std::move(return$.value);");
    }
    writeln!(out, "}}");
}

fn write_return_type(out: &mut OutFile, ty: &Option<Type>) {
    match ty {
        None => write!(out, "void "),
        Some(ty) => write_type_space(out, ty),
    }
}

fn indirect_return(sig: &Signature, types: &Types) -> bool {
    sig.ret
        .as_ref()
        .map_or(false, |ret| sig.throws || types.needs_indirect_abi(ret))
}

fn write_indirect_return_type(out: &mut OutFile, ty: &Type) {
    match ty {
        Type::RustBox(ty) | Type::UniquePtr(ty) => {
            write_type_space(out, &ty.inner);
            write!(out, "*");
        }
        Type::Ref(ty) => {
            if ty.mutability.is_none() {
                write!(out, "const ");
            }
            write_type(out, &ty.inner);
            write!(out, " *");
        }
        Type::Str(_) => write!(out, "::rust::Str::Repr"),
        Type::SliceRefU8(_) => write!(out, "::rust::Slice<uint8_t>::Repr"),
        _ => write_type(out, ty),
    }
}

fn write_indirect_return_type_space(out: &mut OutFile, ty: &Type) {
    write_indirect_return_type(out, ty);
    match ty {
        Type::RustBox(_) | Type::UniquePtr(_) | Type::Ref(_) => {}
        Type::Str(_) | Type::SliceRefU8(_) => write!(out, " "),
        _ => write_space_after_type(out, ty),
    }
}

fn write_extern_return_type_space(out: &mut OutFile, ty: &Option<Type>) {
    match ty {
        Some(Type::RustBox(ty)) | Some(Type::UniquePtr(ty)) => {
            write_type_space(out, &ty.inner);
            write!(out, "*");
        }
        Some(Type::Ref(ty)) => {
            if ty.mutability.is_none() {
                write!(out, "const ");
            }
            write_type(out, &ty.inner);
            write!(out, " *");
        }
        Some(Type::Str(_)) => write!(out, "::rust::Str::Repr "),
        Some(Type::SliceRefU8(_)) => write!(out, "::rust::Slice<uint8_t>::Repr "),
        Some(ty) if out.types.needs_indirect_abi(ty) => write!(out, "void "),
        _ => write_return_type(out, ty),
    }
}

fn write_extern_arg(out: &mut OutFile, arg: &Var) {
    match &arg.ty {
        Type::RustBox(ty) | Type::UniquePtr(ty) | Type::CxxVector(ty) => {
            write_type_space(out, &ty.inner);
            write!(out, "*");
        }
        Type::Str(_) => write!(out, "::rust::Str::Repr "),
        Type::SliceRefU8(_) => write!(out, "::rust::Slice<uint8_t>::Repr "),
        _ => write_type_space(out, &arg.ty),
    }
    if out.types.needs_indirect_abi(&arg.ty) {
        write!(out, "*");
    }
    write!(out, "{}", arg.ident);
}

fn write_type(out: &mut OutFile, ty: &Type) {
    match ty {
        Type::Ident(ident) => match Atom::from(&ident.rust) {
            Some(atom) => write_atom(out, atom),
            None => write!(out, "{}", out.types.resolve(ident).to_fully_qualified()),
        },
        Type::RustBox(ty) => {
            write!(out, "::rust::Box<");
            write_type(out, &ty.inner);
            write!(out, ">");
        }
        Type::RustVec(ty) => {
            write!(out, "::rust::Vec<");
            write_type(out, &ty.inner);
            write!(out, ">");
        }
        Type::UniquePtr(ptr) => {
            write!(out, "::std::unique_ptr<");
            write_type(out, &ptr.inner);
            write!(out, ">");
        }
        Type::CxxVector(ty) => {
            write!(out, "::std::vector<");
            write_type(out, &ty.inner);
            write!(out, ">");
        }
        Type::Ref(r) => {
            if r.mutability.is_none() {
                write!(out, "const ");
            }
            write_type(out, &r.inner);
            write!(out, " &");
        }
        Type::Slice(_) => {
            // For now, only U8 slices are supported, which are covered separately below
            unreachable!()
        }
        Type::Str(_) => {
            write!(out, "::rust::Str");
        }
        Type::SliceRefU8(_) => {
            write!(out, "::rust::Slice<uint8_t>");
        }
        Type::Fn(f) => {
            write!(out, "::rust::{}<", if f.throws { "TryFn" } else { "Fn" });
            match &f.ret {
                Some(ret) => write_type(out, ret),
                None => write!(out, "void"),
            }
            write!(out, "(");
            for (i, arg) in f.args.iter().enumerate() {
                if i > 0 {
                    write!(out, ", ");
                }
                write_type(out, &arg.ty);
            }
            write!(out, ")>");
        }
        Type::Void(_) => unreachable!(),
    }
}

fn write_atom(out: &mut OutFile, atom: Atom) {
    match atom {
        Bool => write!(out, "bool"),
        U8 => write!(out, "uint8_t"),
        U16 => write!(out, "uint16_t"),
        U32 => write!(out, "uint32_t"),
        U64 => write!(out, "uint64_t"),
        Usize => write!(out, "size_t"),
        I8 => write!(out, "int8_t"),
        I16 => write!(out, "int16_t"),
        I32 => write!(out, "int32_t"),
        I64 => write!(out, "int64_t"),
        Isize => write!(out, "::rust::isize"),
        F32 => write!(out, "float"),
        F64 => write!(out, "double"),
        CxxString => write!(out, "::std::string"),
        RustString => write!(out, "::rust::String"),
    }
}

fn write_type_space(out: &mut OutFile, ty: &Type) {
    write_type(out, ty);
    write_space_after_type(out, ty);
}

fn write_space_after_type(out: &mut OutFile, ty: &Type) {
    match ty {
        Type::Ident(_)
        | Type::RustBox(_)
        | Type::UniquePtr(_)
        | Type::Str(_)
        | Type::CxxVector(_)
        | Type::RustVec(_)
        | Type::SliceRefU8(_)
        | Type::Fn(_) => write!(out, " "),
        Type::Ref(_) => {}
        Type::Void(_) | Type::Slice(_) => unreachable!(),
    }
}

// Only called for legal referent types of unique_ptr and element types of
// std::vector and Vec.
fn to_typename(ty: &Type, types: &Types) -> String {
    match ty {
        Type::Ident(ident) => types.resolve(&ident).to_fully_qualified(),
        Type::CxxVector(ptr) => format!("::std::vector<{}>", to_typename(&ptr.inner, types)),
        _ => unreachable!(),
    }
}

// Only called for legal referent types of unique_ptr and element types of
// std::vector and Vec.
fn to_mangled(ty: &Type, types: &Types) -> Symbol {
    match ty {
        Type::Ident(ident) => ident.to_symbol(types),
        Type::CxxVector(ptr) => to_mangled(&ptr.inner, types).prefix_with("std$vector$"),
        _ => unreachable!(),
    }
}

fn write_generic_instantiations(out: &mut OutFile) {
    out.begin_block("extern \"C\"");
    for ty in out.types {
        if let Type::RustBox(ty) = ty {
            if let Type::Ident(inner) = &ty.inner {
                out.next_section();
                write_rust_box_extern(out, &out.types.resolve(&inner));
            }
        } else if let Type::RustVec(ty) = ty {
            if let Type::Ident(inner) = &ty.inner {
                if Atom::from(&inner.rust).is_none() {
                    out.next_section();
                    write_rust_vec_extern(out, inner);
                }
            }
        } else if let Type::UniquePtr(ptr) = ty {
            if let Type::Ident(inner) = &ptr.inner {
                if Atom::from(&inner.rust).is_none()
                    && (!out.types.aliases.contains_key(&inner.rust)
                        || out.types.explicit_impls.contains(ty))
                {
                    out.next_section();
                    write_unique_ptr(out, inner);
                }
            }
        } else if let Type::CxxVector(ptr) = ty {
            if let Type::Ident(inner) = &ptr.inner {
                if Atom::from(&inner.rust).is_none()
                    && (!out.types.aliases.contains_key(&inner.rust)
                        || out.types.explicit_impls.contains(ty))
                {
                    out.next_section();
                    write_cxx_vector(out, ty, inner);
                }
            }
        }
    }
    out.end_block("extern \"C\"");

    out.begin_block("namespace rust");
    out.begin_block("inline namespace cxxbridge05");
    for ty in out.types {
        if let Type::RustBox(ty) = ty {
            if let Type::Ident(inner) = &ty.inner {
                write_rust_box_impl(out, &out.types.resolve(&inner));
            }
        } else if let Type::RustVec(ty) = ty {
            if let Type::Ident(inner) = &ty.inner {
                if Atom::from(&inner.rust).is_none() {
                    write_rust_vec_impl(out, inner);
                }
            }
        }
    }
    out.end_block("namespace cxxbridge05");
    out.end_block("namespace rust");
}

fn write_rust_box_extern(out: &mut OutFile, ident: &CppName) {
    let inner = ident.to_fully_qualified();
    let instance = ident.to_symbol();

    writeln!(out, "#ifndef CXXBRIDGE05_RUST_BOX_{}", instance);
    writeln!(out, "#define CXXBRIDGE05_RUST_BOX_{}", instance);
    writeln!(
        out,
        "void cxxbridge05$box${}$uninit(::rust::Box<{}> *ptr) noexcept;",
        instance, inner,
    );
    writeln!(
        out,
        "void cxxbridge05$box${}$drop(::rust::Box<{}> *ptr) noexcept;",
        instance, inner,
    );
    writeln!(out, "#endif // CXXBRIDGE05_RUST_BOX_{}", instance);
}

fn write_rust_vec_extern(out: &mut OutFile, element: &ResolvableName) {
    let element = Type::Ident(element.clone());
    let inner = to_typename(&element, out.types);
    let instance = to_mangled(&element, out.types);

    writeln!(out, "#ifndef CXXBRIDGE05_RUST_VEC_{}", instance);
    writeln!(out, "#define CXXBRIDGE05_RUST_VEC_{}", instance);
    writeln!(
        out,
        "void cxxbridge05$rust_vec${}$new(const ::rust::Vec<{}> *ptr) noexcept;",
        instance, inner,
    );
    writeln!(
        out,
        "void cxxbridge05$rust_vec${}$drop(::rust::Vec<{}> *ptr) noexcept;",
        instance, inner,
    );
    writeln!(
        out,
        "size_t cxxbridge05$rust_vec${}$len(const ::rust::Vec<{}> *ptr) noexcept;",
        instance, inner,
    );
    writeln!(
        out,
        "const {} *cxxbridge05$rust_vec${}$data(const ::rust::Vec<{0}> *ptr) noexcept;",
        inner, instance,
    );
    writeln!(
        out,
        "size_t cxxbridge05$rust_vec${}$stride() noexcept;",
        instance,
    );
    writeln!(out, "#endif // CXXBRIDGE05_RUST_VEC_{}", instance);
}

fn write_rust_box_impl(out: &mut OutFile, ident: &CppName) {
    let inner = ident.to_fully_qualified();
    let instance = ident.to_symbol();

    writeln!(out, "template <>");
    writeln!(out, "void Box<{}>::uninit() noexcept {{", inner);
    writeln!(out, "  cxxbridge05$box${}$uninit(this);", instance);
    writeln!(out, "}}");

    writeln!(out, "template <>");
    writeln!(out, "void Box<{}>::drop() noexcept {{", inner);
    writeln!(out, "  cxxbridge05$box${}$drop(this);", instance);
    writeln!(out, "}}");
}

fn write_rust_vec_impl(out: &mut OutFile, element: &ResolvableName) {
    let element = Type::Ident(element.clone());
    let inner = to_typename(&element, out.types);
    let instance = to_mangled(&element, out.types);

    writeln!(out, "template <>");
    writeln!(out, "Vec<{}>::Vec() noexcept {{", inner);
    writeln!(out, "  cxxbridge05$rust_vec${}$new(this);", instance);
    writeln!(out, "}}");

    writeln!(out, "template <>");
    writeln!(out, "void Vec<{}>::drop() noexcept {{", inner);
    writeln!(
        out,
        "  return cxxbridge05$rust_vec${}$drop(this);",
        instance,
    );
    writeln!(out, "}}");

    writeln!(out, "template <>");
    writeln!(out, "size_t Vec<{}>::size() const noexcept {{", inner);
    writeln!(out, "  return cxxbridge05$rust_vec${}$len(this);", instance);
    writeln!(out, "}}");

    writeln!(out, "template <>");
    writeln!(out, "const {} *Vec<{0}>::data() const noexcept {{", inner);
    writeln!(
        out,
        "  return cxxbridge05$rust_vec${}$data(this);",
        instance,
    );
    writeln!(out, "}}");

    writeln!(out, "template <>");
    writeln!(out, "size_t Vec<{}>::stride() noexcept {{", inner);
    writeln!(out, "  return cxxbridge05$rust_vec${}$stride();", instance);
    writeln!(out, "}}");
}

fn write_unique_ptr(out: &mut OutFile, ident: &ResolvableName) {
    let ty = Type::Ident(ident.clone());
    let instance = to_mangled(&ty, out.types);

    writeln!(out, "#ifndef CXXBRIDGE05_UNIQUE_PTR_{}", instance);
    writeln!(out, "#define CXXBRIDGE05_UNIQUE_PTR_{}", instance);

    write_unique_ptr_common(out, &ty);

    writeln!(out, "#endif // CXXBRIDGE05_UNIQUE_PTR_{}", instance);
}

// Shared by UniquePtr<T> and UniquePtr<CxxVector<T>>.
fn write_unique_ptr_common(out: &mut OutFile, ty: &Type) {
    out.include.new = true;
    out.include.utility = true;
    let inner = to_typename(ty, out.types);
    let instance = to_mangled(ty, out.types);

    let can_construct_from_value = match ty {
        // Some aliases are to opaque types; some are to trivial types. We can't
        // know at code generation time, so we generate both C++ and Rust side
        // bindings for a "new" method anyway. But the Rust code can't be called
        // for Opaque types because the 'new' method is not implemented.
        Type::Ident(ident) => {
            out.types.structs.contains_key(&ident.rust)
                || out.types.aliases.contains_key(&ident.rust)
        }
        _ => false,
    };

    writeln!(
        out,
        "static_assert(sizeof(::std::unique_ptr<{}>) == sizeof(void *), \"\");",
        inner,
    );
    writeln!(
        out,
        "static_assert(alignof(::std::unique_ptr<{}>) == alignof(void *), \"\");",
        inner,
    );
    writeln!(
        out,
        "void cxxbridge05$unique_ptr${}$null(::std::unique_ptr<{}> *ptr) noexcept {{",
        instance, inner,
    );
    writeln!(out, "  new (ptr) ::std::unique_ptr<{}>();", inner);
    writeln!(out, "}}");
    if can_construct_from_value {
        writeln!(
            out,
            "void cxxbridge05$unique_ptr${}$new(::std::unique_ptr<{}> *ptr, {} *value) noexcept {{",
            instance, inner, inner,
        );
        writeln!(
            out,
            "  new (ptr) ::std::unique_ptr<{}>(new {}(::std::move(*value)));",
            inner, inner,
        );
        writeln!(out, "}}");
    }
    writeln!(
        out,
        "void cxxbridge05$unique_ptr${}$raw(::std::unique_ptr<{}> *ptr, {} *raw) noexcept {{",
        instance, inner, inner,
    );
    writeln!(out, "  new (ptr) ::std::unique_ptr<{}>(raw);", inner);
    writeln!(out, "}}");
    writeln!(
        out,
        "const {} *cxxbridge05$unique_ptr${}$get(const ::std::unique_ptr<{}>& ptr) noexcept {{",
        inner, instance, inner,
    );
    writeln!(out, "  return ptr.get();");
    writeln!(out, "}}");
    writeln!(
        out,
        "{} *cxxbridge05$unique_ptr${}$release(::std::unique_ptr<{}>& ptr) noexcept {{",
        inner, instance, inner,
    );
    writeln!(out, "  return ptr.release();");
    writeln!(out, "}}");
    writeln!(
        out,
        "void cxxbridge05$unique_ptr${}$drop(::std::unique_ptr<{}> *ptr) noexcept {{",
        instance, inner,
    );
    writeln!(out, "  ptr->~unique_ptr();");
    writeln!(out, "}}");
}

fn write_cxx_vector(out: &mut OutFile, vector_ty: &Type, element: &ResolvableName) {
    let element = Type::Ident(element.clone());
    let inner = to_typename(&element, out.types);
    let instance = to_mangled(&element, out.types);

    writeln!(out, "#ifndef CXXBRIDGE05_VECTOR_{}", instance);
    writeln!(out, "#define CXXBRIDGE05_VECTOR_{}", instance);
    writeln!(
        out,
        "size_t cxxbridge05$std$vector${}$size(const ::std::vector<{}> &s) noexcept {{",
        instance, inner,
    );
    writeln!(out, "  return s.size();");
    writeln!(out, "}}");
    writeln!(
        out,
        "const {} *cxxbridge05$std$vector${}$get_unchecked(const ::std::vector<{}> &s, size_t pos) noexcept {{",
        inner, instance, inner,
    );
    writeln!(out, "  return &s[pos];");
    writeln!(out, "}}");

    write_unique_ptr_common(out, vector_ty);

    writeln!(out, "#endif // CXXBRIDGE05_VECTOR_{}", instance);
}
