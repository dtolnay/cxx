use crate::gen::block::Block;
use crate::gen::nested::NamespaceEntries;
use crate::gen::out::OutFile;
use crate::gen::{builtin, include, Opt};
use crate::syntax::atom::Atom::{self, *};
use crate::syntax::symbol::Symbol;
use crate::syntax::{
    mangle, Api, Enum, ExternFn, ExternType, Pair, ResolvableName, Signature, Struct, Type, Types,
    Var,
};
use proc_macro2::Ident;
use std::collections::{HashMap, HashSet};

pub(super) fn gen(apis: &[Api], types: &Types, opt: &Opt, header: bool) -> Vec<u8> {
    let mut out_file = OutFile::new(header, opt, types);
    let out = &mut out_file;

    pick_includes_and_builtins(out, apis);
    out.include.extend(&opt.include);

    write_forward_declarations(out, apis);
    write_data_structures(out, apis);
    write_functions(out, apis);
    write_generic_instantiations(out);

    builtin::write(out);
    include::write(out);

    out_file.content()
}

fn write_forward_declarations(out: &mut OutFile, apis: &[Api]) {
    let needs_forward_declaration = |api: &&Api| match api {
        Api::Struct(_) | Api::CxxType(_) | Api::RustType(_) => true,
        Api::Enum(enm) => !out.types.cxx.contains(&enm.name.rust),
        _ => false,
    };

    let apis_by_namespace =
        NamespaceEntries::new(apis.iter().filter(needs_forward_declaration).collect());

    write(out, &apis_by_namespace, 0);

    fn write(out: &mut OutFile, ns_entries: &NamespaceEntries, indent: usize) {
        let apis = ns_entries.direct_content();

        for api in apis {
            write!(out, "{:1$}", "", indent);
            match api {
                Api::Struct(strct) => write_struct_decl(out, &strct.name.cxx),
                Api::Enum(enm) => write_enum_decl(out, enm),
                Api::CxxType(ety) => write_struct_using(out, &ety.name),
                Api::RustType(ety) => write_struct_decl(out, &ety.name.cxx),
                _ => unreachable!(),
            }
        }

        for (namespace, nested_ns_entries) in ns_entries.nested_content() {
            writeln!(out, "{:2$}namespace {} {{", "", namespace, indent);
            write(out, nested_ns_entries, indent + 2);
            writeln!(out, "{:1$}}}", "", indent);
        }
    }
}

fn write_data_structures<'a>(out: &mut OutFile<'a>, apis: &'a [Api]) {
    let mut methods_for_type = HashMap::new();
    for api in apis {
        if let Api::CxxFunction(efn) | Api::RustFunction(efn) = api {
            if let Some(receiver) = &efn.sig.receiver {
                methods_for_type
                    .entry(&receiver.ty.rust)
                    .or_insert_with(Vec::new)
                    .push(efn);
            }
        }
    }

    let mut structs_written = HashSet::new();
    let mut toposorted_structs = out.types.toposorted_structs.iter();
    for api in apis {
        match api {
            Api::Struct(strct) if !structs_written.contains(&strct.name.rust) => {
                for next in &mut toposorted_structs {
                    if !out.types.cxx.contains(&strct.name.rust) {
                        out.next_section();
                        let methods = methods_for_type
                            .get(&strct.name.rust)
                            .map(Vec::as_slice)
                            .unwrap_or_default();
                        write_struct(out, next, methods);
                    }
                    structs_written.insert(&next.name.rust);
                    if next.name.rust == strct.name.rust {
                        break;
                    }
                }
            }
            Api::Enum(enm) => {
                out.next_section();
                if out.types.cxx.contains(&enm.name.rust) {
                    check_enum(out, enm);
                } else {
                    write_enum(out, enm);
                }
            }
            Api::RustType(ety) => {
                if let Some(methods) = methods_for_type.get(&ety.name.rust) {
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
            if out.types.required_trivial.contains_key(&ety.name.rust) {
                check_trivial_extern_type(out, &ety.name)
            }
        }
    }
}

fn write_functions<'a>(out: &mut OutFile<'a>, apis: &'a [Api]) {
    if !out.header {
        for api in apis {
            match api {
                Api::CxxFunction(efn) => write_cxx_function_shim(out, efn),
                Api::RustFunction(efn) => write_rust_function_decl(out, efn),
                _ => {}
            }
        }
    }

    for api in apis {
        if let Api::RustFunction(efn) = api {
            out.next_section();
            write_rust_function_shim(out, efn);
        }
    }
}

fn pick_includes_and_builtins(out: &mut OutFile, apis: &[Api]) {
    for api in apis {
        if let Api::Include(include) = api {
            out.include.insert(include);
        }
    }

    for ty in out.types {
        match ty {
            Type::Ident(ident) => match Atom::from(&ident.rust) {
                Some(U8) | Some(U16) | Some(U32) | Some(U64) | Some(I8) | Some(I16) | Some(I32)
                | Some(I64) => out.include.cstdint = true,
                Some(Usize) => out.include.cstddef = true,
                Some(Isize) => out.builtin.rust_isize = true,
                Some(CxxString) => out.include.string = true,
                Some(RustString) => out.builtin.rust_string = true,
                Some(Bool) | Some(F32) | Some(F64) | None => {}
            },
            Type::RustBox(_) => out.builtin.rust_box = true,
            Type::RustVec(_) => out.builtin.rust_vec = true,
            Type::UniquePtr(_) => out.include.memory = true,
            Type::Str(_) => out.builtin.rust_str = true,
            Type::CxxVector(_) => out.include.vector = true,
            Type::Fn(_) => out.builtin.rust_fn = true,
            Type::Slice(_) => out.builtin.rust_slice = true,
            Type::SliceRefU8(_) => {
                out.include.cstdint = true;
                out.builtin.rust_slice = true;
            }
            Type::Ref(_) | Type::Void(_) => {}
        }
    }
}

fn write_struct<'a>(out: &mut OutFile<'a>, strct: &'a Struct, methods: &[&ExternFn]) {
    out.set_namespace(&strct.name.namespace);
    let guard = format!("CXXBRIDGE05_STRUCT_{}", strct.name.to_symbol());
    writeln!(out, "#ifndef {}", guard);
    writeln!(out, "#define {}", guard);
    for line in strct.doc.to_string().lines() {
        writeln!(out, "//{}", line);
    }
    writeln!(out, "struct {} final {{", strct.name.cxx);
    for field in &strct.fields {
        write!(out, "  ");
        write_type_space(out, &field.ty);
        writeln!(out, "{};", field.ident);
    }
    if !methods.is_empty() {
        writeln!(out);
    }
    for method in methods {
        write!(out, "  ");
        let sig = &method.sig;
        let local_name = method.name.cxx.to_string();
        write_rust_function_shim_decl(out, &local_name, sig, false);
        writeln!(out, ";");
    }
    writeln!(out, "}};");
    writeln!(out, "#endif // {}", guard);
}

fn write_struct_decl(out: &mut OutFile, ident: &Ident) {
    writeln!(out, "struct {};", ident);
}

fn write_enum_decl(out: &mut OutFile, enm: &Enum) {
    write!(out, "enum class {} : ", enm.name.cxx);
    write_atom(out, enm.repr);
    writeln!(out, ";");
}

fn write_struct_using(out: &mut OutFile, ident: &Pair) {
    writeln!(out, "using {} = {};", ident.cxx, ident.to_fully_qualified());
}

fn write_struct_with_methods<'a>(
    out: &mut OutFile<'a>,
    ety: &'a ExternType,
    methods: &[&ExternFn],
) {
    out.set_namespace(&ety.name.namespace);
    let guard = format!("CXXBRIDGE05_STRUCT_{}", ety.name.to_symbol());
    writeln!(out, "#ifndef {}", guard);
    writeln!(out, "#define {}", guard);
    for line in ety.doc.to_string().lines() {
        writeln!(out, "//{}", line);
    }
    writeln!(out, "struct {} final {{", ety.name.cxx);
    writeln!(out, "  {}() = delete;", ety.name.cxx);
    writeln!(
        out,
        "  {}(const {} &) = delete;",
        ety.name.cxx, ety.name.cxx,
    );
    for method in methods {
        write!(out, "  ");
        let sig = &method.sig;
        let local_name = method.name.cxx.to_string();
        write_rust_function_shim_decl(out, &local_name, sig, false);
        writeln!(out, ";");
    }
    writeln!(out, "}};");
    writeln!(out, "#endif // {}", guard);
}

fn write_enum<'a>(out: &mut OutFile<'a>, enm: &'a Enum) {
    out.set_namespace(&enm.name.namespace);
    let guard = format!("CXXBRIDGE05_ENUM_{}", enm.name.to_symbol());
    writeln!(out, "#ifndef {}", guard);
    writeln!(out, "#define {}", guard);
    for line in enm.doc.to_string().lines() {
        writeln!(out, "//{}", line);
    }
    write!(out, "enum class {} : ", enm.name.cxx);
    write_atom(out, enm.repr);
    writeln!(out, " {{");
    for variant in &enm.variants {
        writeln!(out, "  {} = {},", variant.ident, variant.discriminant);
    }
    writeln!(out, "}};");
    writeln!(out, "#endif // {}", guard);
}

fn check_enum<'a>(out: &mut OutFile<'a>, enm: &'a Enum) {
    out.set_namespace(&enm.name.namespace);
    write!(out, "static_assert(sizeof({}) == sizeof(", enm.name.cxx);
    write_atom(out, enm.repr);
    writeln!(out, "), \"incorrect size\");");
    for variant in &enm.variants {
        write!(out, "static_assert(static_cast<");
        write_atom(out, enm.repr);
        writeln!(
            out,
            ">({}::{}) == {}, \"disagrees with the value in #[cxx::bridge]\");",
            enm.name.cxx, variant.ident, variant.discriminant,
        );
    }
}

fn check_trivial_extern_type(out: &mut OutFile, id: &Pair) {
    // NOTE: The following static assertion is just nice-to-have and not
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
    // them if that were wrong. However, in practice correctly reasoning about
    // the relocatability of C++ types is challenging, particularly if the type
    // definition were to change over time, so for now we add this check.
    //
    // There may be legitimate reasons to opt out of this assertion for support
    // of types that the programmer knows are soundly Rust-movable despite not
    // being recognized as such by the C++ type system due to a move constructor
    // or destructor. To opt out of the relocatability check, they need to do
    // one of the following things in any header used by `include!` in their
    // bridge.
    //
    //      --- if they define the type:
    //      struct MyType {
    //        ...
    //    +   using IsRelocatable = std::true_type;
    //      };
    //
    //      --- otherwise:
    //    + template <>
    //    + struct rust::IsRelocatable<MyType> : std::true_type {};
    //

    let id = id.to_fully_qualified();
    out.builtin.relocatable = true;
    writeln!(out, "static_assert(");
    writeln!(out, "    ::rust::IsRelocatable<{}>::value,", id);
    writeln!(
        out,
        "    \"type {} marked as Trivial in Rust is not trivially move constructible and trivially destructible in C++\");",
        id,
    );
}

fn write_cxx_function_shim<'a>(out: &mut OutFile<'a>, efn: &'a ExternFn) {
    out.next_section();
    out.set_namespace(&efn.name.namespace);
    out.begin_block(Block::ExternC);
    if let Some(annotation) = &out.opt.cxx_impl_annotations {
        write!(out, "{} ", annotation);
    }
    if efn.throws {
        out.builtin.ptr_len = true;
        write!(out, "::rust::repr::PtrLen ");
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
            out.types.resolve(&receiver.ty).to_fully_qualified(),
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
        None => write!(out, "(*{}$)(", efn.name.rust),
        Some(receiver) => write!(
            out,
            "({}::*{}$)(",
            out.types.resolve(&receiver.ty).to_fully_qualified(),
            efn.name.rust,
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
        None => write!(out, "{}", efn.name.to_fully_qualified()),
        Some(receiver) => write!(
            out,
            "&{}::{}",
            out.types.resolve(&receiver.ty).to_fully_qualified(),
            efn.name.cxx,
        ),
    }
    writeln!(out, ";");
    write!(out, "  ");
    if efn.throws {
        out.builtin.ptr_len = true;
        out.builtin.trycatch = true;
        writeln!(out, "::rust::repr::PtrLen throw$;");
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
        Some(Type::Str(_)) if !indirect_return => {
            out.builtin.rust_str_repr = true;
            write!(out, "::rust::impl<::rust::Str>::repr(");
        }
        Some(Type::SliceRefU8(_)) if !indirect_return => {
            out.builtin.rust_slice_repr = true;
            write!(out, "::rust::impl<::rust::Slice<uint8_t>>::repr(")
        }
        _ => {}
    }
    match &efn.receiver {
        None => write!(out, "{}$(", efn.name.rust),
        Some(_) => write!(out, "(self.*{}$)(", efn.name.rust),
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
        } else if let Type::Str(_) = arg.ty {
            out.builtin.rust_str_new_unchecked = true;
            write!(
                out,
                "::rust::impl<::rust::Str>::new_unchecked({})",
                arg.ident,
            );
        } else if arg.ty == RustString {
            out.builtin.unsafe_bitcopy = true;
            write!(
                out,
                "::rust::String(::rust::unsafe_bitcopy, *{})",
                arg.ident,
            );
        } else if let Type::RustVec(_) = arg.ty {
            out.builtin.unsafe_bitcopy = true;
            write_type(out, &arg.ty);
            write!(out, "(::rust::unsafe_bitcopy, *{})", arg.ident);
        } else if let Type::SliceRefU8(_) = arg.ty {
            write!(
                out,
                "::rust::Slice<uint8_t>(static_cast<const uint8_t *>({0}.ptr), {0}.len)",
                arg.ident,
            );
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
        out.builtin.exception = true;
        writeln!(out, "        throw$.ptr = nullptr;");
        writeln!(out, "      }},");
        writeln!(out, "      [&](const char *catch$) noexcept {{");
        writeln!(out, "        throw$.len = ::std::strlen(catch$);");
        writeln!(
            out,
            "        throw$.ptr = ::cxxbridge05$exception(catch$, throw$.len);",
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
    out.end_block(Block::ExternC);
}

fn write_function_pointer_trampoline(
    out: &mut OutFile,
    efn: &ExternFn,
    var: &Ident,
    f: &Signature,
) {
    let r_trampoline = mangle::r_trampoline(efn, var, out.types);
    let indirect_call = true;
    write_rust_function_decl_impl(out, &r_trampoline, f, indirect_call);

    out.next_section();
    let c_trampoline = mangle::c_trampoline(efn, var, out.types).to_string();
    write_rust_function_shim_impl(out, &c_trampoline, f, &r_trampoline, indirect_call);
}

fn write_rust_function_decl<'a>(out: &mut OutFile<'a>, efn: &'a ExternFn) {
    out.set_namespace(&efn.name.namespace);
    out.begin_block(Block::ExternC);
    let link_name = mangle::extern_fn(efn, out.types);
    let indirect_call = false;
    write_rust_function_decl_impl(out, &link_name, efn, indirect_call);
    out.end_block(Block::ExternC);
}

fn write_rust_function_decl_impl(
    out: &mut OutFile,
    link_name: &Symbol,
    sig: &Signature,
    indirect_call: bool,
) {
    out.next_section();
    if sig.throws {
        out.builtin.ptr_len = true;
        write!(out, "::rust::repr::PtrLen ");
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
            out.types.resolve(&receiver.ty).to_fully_qualified(),
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

fn write_rust_function_shim<'a>(out: &mut OutFile<'a>, efn: &'a ExternFn) {
    out.set_namespace(&efn.name.namespace);
    for line in efn.doc.to_string().lines() {
        writeln!(out, "//{}", line);
    }
    let local_name = match &efn.sig.receiver {
        None => efn.name.cxx.to_string(),
        Some(receiver) => format!("{}::{}", out.types.resolve(&receiver.ty).cxx, efn.name.cxx),
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
            out.builtin.manually_drop = true;
            write!(out, "  ::rust::ManuallyDrop<");
            write_type(out, &arg.ty);
            writeln!(out, "> {}$(::std::move({0}));", arg.ident);
        }
    }
    write!(out, "  ");
    let indirect_return = indirect_return(sig, out.types);
    if indirect_return {
        out.builtin.maybe_uninit = true;
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
            Type::Str(_) => {
                out.builtin.rust_str_new_unchecked = true;
                write!(out, "::rust::impl<::rust::Str>::new_unchecked(");
            }
            Type::SliceRefU8(_) => {
                out.builtin.rust_slice_new = true;
                write!(out, "::rust::impl<::rust::Slice<uint8_t>>::slice(");
            }
            _ => {}
        }
    }
    if sig.throws {
        out.builtin.ptr_len = true;
        write!(out, "::rust::repr::PtrLen error$ = ");
    }
    write!(out, "{}(", invoke);
    let mut needs_comma = false;
    if sig.receiver.is_some() {
        write!(out, "*this");
        needs_comma = true;
    }
    for arg in &sig.args {
        if needs_comma {
            write!(out, ", ");
        }
        match &arg.ty {
            Type::Str(_) => {
                out.builtin.rust_str_repr = true;
                write!(out, "::rust::impl<::rust::Str>::repr(");
            }
            Type::SliceRefU8(_) => {
                out.builtin.rust_slice_repr = true;
                write!(out, "::rust::impl<::rust::Slice<uint8_t>>::repr(");
            }
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
        needs_comma = true;
    }
    if indirect_return {
        if needs_comma {
            write!(out, ", ");
        }
        write!(out, "&return$.value");
        needs_comma = true;
    }
    if indirect_call {
        if needs_comma {
            write!(out, ", ");
        }
        write!(out, "extern$");
    }
    write!(out, ")");
    if !indirect_return {
        if let Some(ret) = &sig.ret {
            if let Type::RustBox(_) | Type::UniquePtr(_) | Type::Str(_) | Type::SliceRefU8(_) = ret
            {
                write!(out, ")");
            }
        }
    }
    writeln!(out, ";");
    if sig.throws {
        out.builtin.rust_error = true;
        writeln!(out, "  if (error$.ptr) {{");
        writeln!(out, "    throw ::rust::impl<::rust::Error>::error(error$);");
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
        Some(Type::Str(_)) | Some(Type::SliceRefU8(_)) => {
            out.builtin.ptr_len = true;
            write!(out, "::rust::repr::PtrLen ");
        }
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
        Type::Str(_) | Type::SliceRefU8(_) => {
            out.builtin.ptr_len = true;
            write!(out, "::rust::repr::PtrLen ");
        }
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
    if out.header {
        return;
    }

    out.next_section();
    out.set_namespace(Default::default());
    out.begin_block(Block::ExternC);
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
    out.end_block(Block::ExternC);

    out.begin_block(Block::Namespace("rust"));
    out.begin_block(Block::InlineNamespace("cxxbridge05"));
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
    out.end_block(Block::InlineNamespace("cxxbridge05"));
    out.end_block(Block::Namespace("rust"));
}

fn write_rust_box_extern(out: &mut OutFile, ident: &Pair) {
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
        "void cxxbridge05$rust_vec${}$reserve_total(::rust::Vec<{}> *ptr, size_t cap) noexcept;",
        instance, inner,
    );
    writeln!(
        out,
        "void cxxbridge05$rust_vec${}$set_len(::rust::Vec<{}> *ptr, size_t len) noexcept;",
        instance, inner,
    );
    writeln!(
        out,
        "size_t cxxbridge05$rust_vec${}$stride() noexcept;",
        instance,
    );
    writeln!(out, "#endif // CXXBRIDGE05_RUST_VEC_{}", instance);
}

fn write_rust_box_impl(out: &mut OutFile, ident: &Pair) {
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
    writeln!(
        out,
        "void Vec<{}>::reserve_total(size_t cap) noexcept {{",
        inner,
    );
    writeln!(
        out,
        "  return cxxbridge05$rust_vec${}$reserve_total(this, cap);",
        instance,
    );
    writeln!(out, "}}");

    writeln!(out, "template <>");
    writeln!(out, "void Vec<{}>::set_len(size_t len) noexcept {{", inner);
    writeln!(
        out,
        "  return cxxbridge05$rust_vec${}$set_len(this, len);",
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
