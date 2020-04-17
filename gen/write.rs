use crate::gen::namespace::Namespace;
use crate::gen::out::OutFile;
use crate::gen::{include, Opt};
use crate::syntax::atom::Atom::{self, *};
use crate::syntax::{Api, ExternFn, ExternType, Receiver, Signature, Struct, Type, Types, Var};
use itertools::Itertools;
use proc_macro2::Ident;

pub(super) fn gen(
    namespace: Namespace,
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
            out.include.insert(include.value());
        }
    }

    write_includes(out, types);
    write_include_cxxbridge(out, apis, types);

    out.next_section();
    for name in &namespace {
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

    let methods_for_type = apis.iter().filter_map(|api| match api {
        Api::RustFunction(efn) => match &efn.sig.receiver {
            Some(rcvr) => Some((&rcvr.ident, efn)),
            _ => None,
        },
        _ => None,
    }).into_group_map();

    for api in apis {
        match api {
            Api::Struct(strct) => {
                out.next_section();
                write_struct(out, strct);
            }
            Api::RustType(ety) => {
                match methods_for_type.get(&ety.ident) {
                    Some(methods) => {
                        out.next_section();
                        write_struct_with_methods(out, ety, methods);
                    },
                    _ => {}
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
    let mut needs_rust_fn = false;
    let mut needs_rust_isize = false;
    for ty in types {
        match ty {
            Type::RustBox(_) => {
                out.include.type_traits = true;
                needs_rust_box = true;
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
                out.include.base_tsd = true;
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
                    if arg.ty == RustString {
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
    out.begin_block("inline namespace cxxbridge02");

    if needs_rust_string
        || needs_rust_str
        || needs_rust_slice
        || needs_rust_box
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

    if needs_rust_string {
        out.next_section();
        writeln!(out, "struct unsafe_bitcopy_t;");
    }

    write_header_section(out, needs_rust_string, "CXXBRIDGE02_RUST_STRING");
    write_header_section(out, needs_rust_str, "CXXBRIDGE02_RUST_STR");
    write_header_section(out, needs_rust_slice, "CXXBRIDGE02_RUST_SLICE");
    write_header_section(out, needs_rust_box, "CXXBRIDGE02_RUST_BOX");
    write_header_section(out, needs_rust_fn, "CXXBRIDGE02_RUST_FN");
    write_header_section(out, needs_rust_error, "CXXBRIDGE02_RUST_ERROR");
    write_header_section(out, needs_rust_isize, "CXXBRIDGE02_RUST_ISIZE");
    write_header_section(out, needs_unsafe_bitcopy, "CXXBRIDGE02_RUST_BITCOPY");

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

    out.end_block("namespace cxxbridge02");

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

fn write_header_section(out: &mut OutFile, needed: bool, section: &str) {
    let section = include::get(section);
    if needed {
        out.next_section();
        for line in section.lines() {
            if !line.trim_start().starts_with("//") {
                writeln!(out, "{}", line);
            }
        }
    }
}

fn write_struct(out: &mut OutFile, strct: &Struct) {
    for line in strct.doc.to_string().lines() {
        writeln!(out, "//{}", line);
    }
    writeln!(out, "struct {} final {{", strct.ident);
    for field in &strct.fields {
        write!(out, "  ");
        write_type_space(out, &field.ty);
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

fn write_struct_with_methods(out: &mut OutFile, ety: &ExternType, methods: &Vec<&ExternFn>) {
    for line in ety.doc.to_string().lines() {
        writeln!(out, "//{}", line);
    }
    writeln!(out, "struct {} final {{", ety.ident);
    writeln!(out, "  {}() = delete;", ety.ident);
    writeln!(out, "  {}(const {}&) = delete;", ety.ident, ety.ident);
    for method in methods {
        write!(out, "  ");
        let sig = &method.sig;
        let local_name = method.ident.to_string();
        write_rust_function_shim_decl(out, &local_name, sig, None, false);
        writeln!(out, ";");
    }
    writeln!(out, "}};");
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
            "const char *cxxbridge02$exception(const char *, size_t);",
        );
    }
}

fn write_cxx_function_shim(out: &mut OutFile, efn: &ExternFn, types: &Types) {
    if efn.throws {
        write!(out, "::rust::Str::Repr ");
    } else {
        write_extern_return_type_space(out, &efn.ret, types);
    }
    let receiver_type = match &efn.receiver {
        Some(base) => base.ident.to_string(),
        None => "_".to_string(),
    };
    write!(out, "{}cxxbridge02${}${}(", out.namespace, receiver_type, efn.ident);
    if let Some(base) = &efn.receiver {
        write!(out, "{} *__receiver$", base.ident);
    }
    for (i, arg) in efn.args.iter().enumerate() {
        if i > 0 || efn.receiver.is_some() {
            write!(out, ", ");
        }
        if arg.ty == RustString {
            write!(out, "const ");
        }
        write_extern_arg(out, arg, types);
    }
    let indirect_return = indirect_return(efn, types);
    if indirect_return {
        if !efn.args.is_empty() {
            write!(out, ", ");
        }
        write_indirect_return_type_space(out, efn.ret.as_ref().unwrap());
        write!(out, "*return$");
    }
    writeln!(out, ") noexcept {{");
    write!(out, "  ");
    write_return_type(out, &efn.ret);
    match &efn.receiver {
        None => write!(out, "(*{}$)(", efn.ident),
        Some(base) => write!(out, "({}::*{}$)(", base.ident, efn.ident),
    }
    for (i, arg) in efn.args.iter().enumerate() {
        if i > 0 {
            write!(out, ", ");
        }
        write_type(out, &arg.ty);
    }
    write!(out, ")");
    match &efn.receiver {
        Some(Receiver { mutability: None, ident: _ }) => write!(out, " const"),
        _ => {},
    }
    write!(out, " = ");
    match &efn.receiver {
        None => write!(out, "{}", efn.ident),
        Some(base) => write!(out, "&{}::{}", base.ident, efn.ident),
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
        None => write!(out, "{}$(", efn.ident),
        Some(_) => write!(out, "(__receiver$->*{}$)(", efn.ident),
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
        } else if types.needs_indirect_abi(&arg.ty) {
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
            "        throw$.ptr = cxxbridge02$exception(catch$, throw$.len);",
        );
        writeln!(out, "      }});");
        writeln!(out, "  return throw$;");
    }
    writeln!(out, "}}");
    for arg in &efn.args {
        if let Type::Fn(f) = &arg.ty {
            let var = &arg.ident;
            write_function_pointer_trampoline(out, efn, var, f, types);
        }
    }
}

fn write_function_pointer_trampoline(
    out: &mut OutFile,
    efn: &ExternFn,
    var: &Ident,
    f: &Signature,
    types: &Types,
) {
    out.next_section();
    let r_trampoline = format!("{}cxxbridge02${}${}$1", out.namespace, efn.ident, var);
    let indirect_call = true;
    write_rust_function_decl_impl(out, &r_trampoline, f, types, indirect_call);

    out.next_section();
    let c_trampoline = format!("{}cxxbridge02${}${}$0", out.namespace, efn.ident, var);
    write_rust_function_shim_impl(out, &c_trampoline, f, types, &r_trampoline, indirect_call);
}

fn write_rust_function_decl(out: &mut OutFile, efn: &ExternFn, types: &Types) {
    let receiver_type = match &efn.receiver {
        Some(base) => base.ident.to_string(),
        None => "_".to_string(),
    };
    let link_name = format!("{}cxxbridge02${}${}", out.namespace, receiver_type, efn.ident);
    let indirect_call = false;
    write_rust_function_decl_impl(out, &link_name, efn, types, indirect_call);
}

fn write_rust_function_decl_impl(
    out: &mut OutFile,
    link_name: &str,
    sig: &Signature,
    types: &Types,
    indirect_call: bool,
) {
    if sig.throws {
        write!(out, "::rust::Str::Repr ");
    } else {
        write_extern_return_type_space(out, &sig.ret, types);
    }
    write!(out, "{}(", link_name);
    let mut needs_comma = false;
    if let Some(base) = &sig.receiver {
        write!(out, "{} &__receiver$", base.ident);
        needs_comma = true;
    }
    for arg in &sig.args {
        if needs_comma {
            write!(out, ", ");
        }
        write_extern_arg(out, arg, types);
        needs_comma = true;
    }
    if indirect_return(sig, types) {
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

fn write_rust_function_shim(out: &mut OutFile, efn: &ExternFn, types: &Types) {
    for line in efn.doc.to_string().lines() {
        writeln!(out, "//{}", line);
    }
    let local_name = efn.ident.to_string();
    let receiver_type = match &efn.receiver {
        Some(base) => base.ident.to_string(),
        None => "_".to_string(),
    };
    let invoke = format!("{}cxxbridge02${}${}", out.namespace, receiver_type, efn.ident);
    let indirect_call = false;
    write_rust_function_shim_impl(out, &local_name, efn, types, &invoke, indirect_call);
}

fn write_rust_function_shim_decl(
    out: &mut OutFile,
    local_name: &str,
    sig: &Signature,
    receiver: Option<&Receiver>,
    indirect_call: bool,
) {
    write_return_type(out, &sig.ret);
    if let Some(base) = receiver {
        write!(out, "{}::", base.ident);
    }
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
    if !sig.throws {
        write!(out, " noexcept");
    }
}

fn write_rust_function_shim_impl(
    out: &mut OutFile,
    local_name: &str,
    sig: &Signature,
    types: &Types,
    invoke: &str,
    indirect_call: bool,
) {
    if out.header && sig.receiver.is_some() {
        // We've already defined this inside the struct.
        return;
    }
    write_rust_function_shim_decl(out, local_name, sig, sig.receiver.as_ref(), indirect_call);
    if out.header {
        writeln!(out, ";");
    } else {
        writeln!(out, " {{");
        for arg in &sig.args {
            if arg.ty != RustString && types.needs_indirect_abi(&arg.ty) {
                out.include.utility = true;
                write!(out, "  ::rust::ManuallyDrop<");
                write_type(out, &arg.ty);
                writeln!(out, "> {}$(::std::move({0}));", arg.ident);
            }
        }
        write!(out, "  ");
        let indirect_return = indirect_return(sig, types);
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
        if let Some(_) = &sig.receiver {
            write!(out, "*this");
        }
        for (i, arg) in sig.args.iter().enumerate() {
            if i > 0 || sig.receiver.is_some() {
                write!(out, ", ");
            }
            match &arg.ty {
                Type::Str(_) => write!(out, "::rust::Str::Repr("),
                Type::SliceRefU8(_) => write!(out, "::rust::Slice<uint8_t>::Repr("),
                ty if types.needs_indirect_abi(ty) => write!(out, "&"),
                _ => {}
            }
            write!(out, "{}", arg.ident);
            match &arg.ty {
                Type::RustBox(_) => write!(out, ".into_raw()"),
                Type::UniquePtr(_) => write!(out, ".release()"),
                Type::Str(_) | Type::SliceRefU8(_) => write!(out, ")"),
                ty if ty != RustString && types.needs_indirect_abi(ty) => write!(out, "$.value"),
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
        if let Some(ret) = &sig.ret {
            if let Type::RustBox(_) | Type::UniquePtr(_) = ret {
                write!(out, ")");
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

fn write_extern_return_type_space(out: &mut OutFile, ty: &Option<Type>, types: &Types) {
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
        Some(ty) if types.needs_indirect_abi(ty) => write!(out, "void "),
        _ => write_return_type(out, ty),
    }
}

fn write_extern_arg(out: &mut OutFile, arg: &Var, types: &Types) {
    match &arg.ty {
        Type::RustBox(ty) | Type::UniquePtr(ty) => {
            write_type_space(out, &ty.inner);
            write!(out, "*");
        }
        Type::Str(_) => write!(out, "::rust::Str::Repr "),
        Type::SliceRefU8(_) => write!(out, "::rust::Slice<uint8_t>::Repr "),
        _ => write_type_space(out, &arg.ty),
    }
    if types.needs_indirect_abi(&arg.ty) {
        write!(out, "*");
    }
    write!(out, "{}", arg.ident);
}

fn write_type(out: &mut OutFile, ty: &Type) {
    match ty {
        Type::Ident(ident) => match Atom::from(ident) {
            Some(Bool) => write!(out, "bool"),
            Some(U8) => write!(out, "uint8_t"),
            Some(U16) => write!(out, "uint16_t"),
            Some(U32) => write!(out, "uint32_t"),
            Some(U64) => write!(out, "uint64_t"),
            Some(Usize) => write!(out, "size_t"),
            Some(I8) => write!(out, "int8_t"),
            Some(I16) => write!(out, "int16_t"),
            Some(I32) => write!(out, "int32_t"),
            Some(I64) => write!(out, "int64_t"),
            Some(Isize) => write!(out, "::rust::isize"),
            Some(F32) => write!(out, "float"),
            Some(F64) => write!(out, "double"),
            Some(CxxString) => write!(out, "::std::string"),
            Some(RustString) => write!(out, "::rust::String"),
            None => write!(out, "{}", ident),
        },
        Type::RustBox(ty) => {
            write!(out, "::rust::Box<");
            write_type(out, &ty.inner);
            write!(out, ">");
        }
        Type::UniquePtr(ptr) => {
            write!(out, "::std::unique_ptr<");
            write_type(out, &ptr.inner);
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
        | Type::SliceRefU8(_)
        | Type::Fn(_) => write!(out, " "),
        Type::Ref(_) => {}
        Type::Void(_) | Type::Slice(_) => unreachable!(),
    }
}

fn write_generic_instantiations(out: &mut OutFile, types: &Types) {
    fn allow_unique_ptr(ident: &Ident) -> bool {
        Atom::from(ident).is_none()
    }

    out.begin_block("extern \"C\"");
    for ty in types {
        if let Type::RustBox(ty) = ty {
            if let Type::Ident(inner) = &ty.inner {
                out.next_section();
                write_rust_box_extern(out, inner);
            }
        } else if let Type::UniquePtr(ptr) = ty {
            if let Type::Ident(inner) = &ptr.inner {
                if allow_unique_ptr(inner) {
                    out.next_section();
                    write_unique_ptr(out, inner, types);
                }
            }
        }
    }
    out.end_block("extern \"C\"");

    out.begin_block("namespace rust");
    out.begin_block("inline namespace cxxbridge02");
    for ty in types {
        if let Type::RustBox(ty) = ty {
            if let Type::Ident(inner) = &ty.inner {
                write_rust_box_impl(out, inner);
            }
        }
    }
    out.end_block("namespace cxxbridge02");
    out.end_block("namespace rust");
}

fn write_rust_box_extern(out: &mut OutFile, ident: &Ident) {
    let mut inner = String::new();
    for name in &out.namespace {
        inner += name;
        inner += "::";
    }
    inner += &ident.to_string();
    let instance = inner.replace("::", "$");

    writeln!(out, "#ifndef CXXBRIDGE02_RUST_BOX_{}", instance);
    writeln!(out, "#define CXXBRIDGE02_RUST_BOX_{}", instance);
    writeln!(
        out,
        "void cxxbridge02$box${}$uninit(::rust::Box<{}> *ptr) noexcept;",
        instance, inner,
    );
    writeln!(
        out,
        "void cxxbridge02$box${}$drop(::rust::Box<{}> *ptr) noexcept;",
        instance, inner,
    );
    writeln!(out, "#endif // CXXBRIDGE02_RUST_BOX_{}", instance);
}

fn write_rust_box_impl(out: &mut OutFile, ident: &Ident) {
    let mut inner = String::new();
    for name in &out.namespace {
        inner += name;
        inner += "::";
    }
    inner += &ident.to_string();
    let instance = inner.replace("::", "$");

    writeln!(out, "template <>");
    writeln!(out, "void Box<{}>::uninit() noexcept {{", inner);
    writeln!(out, "  cxxbridge02$box${}$uninit(this);", instance);
    writeln!(out, "}}");

    writeln!(out, "template <>");
    writeln!(out, "void Box<{}>::drop() noexcept {{", inner);
    writeln!(out, "  cxxbridge02$box${}$drop(this);", instance);
    writeln!(out, "}}");
}

fn write_unique_ptr(out: &mut OutFile, ident: &Ident, types: &Types) {
    out.include.utility = true;

    let mut inner = String::new();
    for name in &out.namespace {
        inner += name;
        inner += "::";
    }
    inner += &ident.to_string();
    let instance = inner.replace("::", "$");

    writeln!(out, "#ifndef CXXBRIDGE02_UNIQUE_PTR_{}", instance);
    writeln!(out, "#define CXXBRIDGE02_UNIQUE_PTR_{}", instance);
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
        "void cxxbridge02$unique_ptr${}$null(::std::unique_ptr<{}> *ptr) noexcept {{",
        instance, inner,
    );
    writeln!(out, "  new (ptr) ::std::unique_ptr<{}>();", inner);
    writeln!(out, "}}");
    if types.structs.contains_key(ident) {
        writeln!(
            out,
            "void cxxbridge02$unique_ptr${}$new(::std::unique_ptr<{}> *ptr, {} *value) noexcept {{",
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
        "void cxxbridge02$unique_ptr${}$raw(::std::unique_ptr<{}> *ptr, {} *raw) noexcept {{",
        instance, inner, inner,
    );
    writeln!(out, "  new (ptr) ::std::unique_ptr<{}>(raw);", inner);
    writeln!(out, "}}");
    writeln!(
        out,
        "const {} *cxxbridge02$unique_ptr${}$get(const ::std::unique_ptr<{}>& ptr) noexcept {{",
        inner, instance, inner,
    );
    writeln!(out, "  return ptr.get();");
    writeln!(out, "}}");
    writeln!(
        out,
        "{} *cxxbridge02$unique_ptr${}$release(::std::unique_ptr<{}>& ptr) noexcept {{",
        inner, instance, inner,
    );
    writeln!(out, "  return ptr.release();");
    writeln!(out, "}}");
    writeln!(
        out,
        "void cxxbridge02$unique_ptr${}$drop(::std::unique_ptr<{}> *ptr) noexcept {{",
        instance, inner,
    );
    writeln!(out, "  ptr->~unique_ptr();");
    writeln!(out, "}}");
    writeln!(out, "#endif // CXXBRIDGE02_UNIQUE_PTR_{}", instance);
}
