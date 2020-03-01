use crate::gen::include;
use crate::gen::out::OutFile;
use crate::syntax::atom::Atom::{self, *};
use crate::syntax::{Api, ExternFn, Struct, Type, Types, Var};
use proc_macro2::Ident;

pub(super) fn gen(namespace: Vec<String>, apis: &[Api], types: &Types, header: bool) -> OutFile {
    let mut out_file = OutFile::new(namespace.clone(), header);
    let out = &mut out_file;

    if header {
        writeln!(out, "#pragma once");
    }

    for api in apis {
        if let Api::Include(include) = api {
            writeln!(out, "#include \"{}\"", include.value().escape_default());
        }
    }

    write_includes(out, types);
    write_include_cxxbridge(out, types);

    if !header {
        out.next_section();
        write_namespace_alias(out, types);
    }

    out.next_section();
    for name in &namespace {
        writeln!(out, "namespace {} {{", name);
    }

    if header {
        out.next_section();
        write_namespace_alias(out, types);
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

    for api in apis {
        if let Api::Struct(strct) = api {
            out.next_section();
            write_struct(out, strct);
        }
    }

    if !header {
        out.begin_block("extern \"C\"");
        for api in apis {
            let (efn, write): (_, fn(_, _, _)) = match api {
                Api::CxxFunction(efn) => (efn, write_cxx_function_shim),
                Api::RustFunction(efn) => (efn, write_rust_function_decl),
                _ => continue,
            };
            out.next_section();
            write(out, efn, types);
        }
        out.end_block();
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

    out_file
}

fn write_includes(out: &mut OutFile, types: &Types) {
    let mut has_int = false;
    let mut has_unique_ptr = false;
    let mut has_string = false;

    for ty in types {
        match ty {
            Type::Ident(ident) => match Atom::from(ident) {
                Some(U8) | Some(U16) | Some(U32) | Some(U64) | Some(Usize) | Some(I8)
                | Some(I16) | Some(I32) | Some(I64) | Some(Isize) => has_int = true,
                Some(CxxString) => has_string = true,
                Some(Bool) | Some(RustString) | None => {}
            },
            Type::UniquePtr(_) => has_unique_ptr = true,
            _ => {}
        }
    }

    if has_int {
        writeln!(out, "#include <cstdint>");
    }
    if has_unique_ptr {
        writeln!(out, "#include <memory>");
    }
    if has_string {
        writeln!(out, "#include <string>");
    }
}

fn write_include_cxxbridge(out: &mut OutFile, types: &Types) {
    let mut needs_rust_box = false;
    for ty in types {
        if let Type::RustBox(_) = ty {
            needs_rust_box = true;
            break;
        }
    }

    out.begin_block("namespace cxxbridge01");
    if needs_rust_box {
        writeln!(out, "// #include \"cxxbridge.h\"");
        for line in include::get("CXXBRIDGE01_RUST_BOX").lines() {
            if !line.trim_start().starts_with("//") {
                writeln!(out, "{}", line);
            }
        }
    }
    out.end_block();
}

fn write_namespace_alias(out: &mut OutFile, types: &Types) {
    let mut needs_namespace_alias = false;
    for ty in types {
        if let Type::RustBox(_) = ty {
            needs_namespace_alias = true;
            break;
        }
    }

    if needs_namespace_alias {
        writeln!(out, "namespace cxxbridge = cxxbridge01;");
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

fn write_cxx_function_shim(out: &mut OutFile, efn: &ExternFn, types: &Types) {
    let indirect_return = efn
        .ret
        .as_ref()
        .map_or(false, |ret| types.needs_indirect_abi(ret));
    write_extern_return_type(out, &efn.ret, types);
    for name in out.namespace.clone() {
        write!(out, "{}$", name);
    }
    write!(out, "cxxbridge01${}(", efn.ident);
    for (i, arg) in efn.args.iter().enumerate() {
        if i > 0 {
            write!(out, ", ");
        }
        write_extern_arg(out, arg, types);
    }
    if indirect_return {
        if !efn.args.is_empty() {
            write!(out, ", ");
        }
        write_return_type(out, &efn.ret);
        write!(out, "*return$");
    }
    writeln!(out, ") noexcept {{");
    write!(out, "  ");
    write_return_type(out, &efn.ret);
    write!(out, "(*{}$)(", efn.ident);
    for (i, arg) in efn.args.iter().enumerate() {
        if i > 0 {
            write!(out, ", ");
        }
        write_type(out, &arg.ty);
    }
    writeln!(out, ") = {};", efn.ident);
    write!(out, "  ");
    if indirect_return {
        write!(out, "new (return$) ");
        write_type(out, efn.ret.as_ref().unwrap());
        write!(out, "(");
    } else if let Some(ret) = &efn.ret {
        write!(out, "return ");
        if let Type::Ref(_) = ret {
            write!(out, "&");
        }
    }
    write!(out, "{}$(", efn.ident);
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
        } else if types.needs_indirect_abi(&arg.ty) {
            write!(out, "::std::move(*{})", arg.ident);
        } else {
            write!(out, "{}", arg.ident);
        }
    }
    write!(out, ")");
    match &efn.ret {
        Some(Type::RustBox(_)) => write!(out, ".into_raw()"),
        Some(Type::UniquePtr(_)) => write!(out, ".release()"),
        _ => {}
    }
    if indirect_return {
        write!(out, ")");
    }
    writeln!(out, ";");
    writeln!(out, "}}");
}

fn write_rust_function_decl(out: &mut OutFile, efn: &ExternFn, types: &Types) {
    write_extern_return_type(out, &efn.ret, types);
    for name in out.namespace.clone() {
        write!(out, "{}$", name);
    }
    write!(out, "cxxbridge01${}(", efn.ident);
    for (i, arg) in efn.args.iter().enumerate() {
        if i > 0 {
            write!(out, ", ");
        }
        write_extern_arg(out, arg, types);
    }
    if efn
        .ret
        .as_ref()
        .map_or(false, |ret| types.needs_indirect_abi(ret))
    {
        if !efn.args.is_empty() {
            write!(out, ", ");
        }
        write_return_type(out, &efn.ret);
        write!(out, "*return$");
    }
    writeln!(out, ") noexcept;");
}

fn write_rust_function_shim(out: &mut OutFile, efn: &ExternFn, types: &Types) {
    let indirect_return = efn
        .ret
        .as_ref()
        .map_or(false, |ret| types.needs_indirect_abi(ret));
    for line in efn.doc.to_string().lines() {
        writeln!(out, "//{}", line);
    }
    write_return_type(out, &efn.ret);
    write!(out, "{}(", efn.ident);
    for (i, arg) in efn.args.iter().enumerate() {
        if i > 0 {
            write!(out, ", ");
        }
        write_type_space(out, &arg.ty);
        write!(out, "{}", arg.ident);
    }
    write!(out, ") noexcept");
    if out.header {
        writeln!(out, ";");
    } else {
        writeln!(out, " {{");
        write!(out, "  ");
        if indirect_return {
            write!(out, "char return$[sizeof(");
            write_type(out, efn.ret.as_ref().unwrap());
            writeln!(out, ")];");
            write!(out, "  ");
        } else if let Some(ret) = &efn.ret {
            write!(out, "return ");
            if let Type::Ref(_) = ret {
                write!(out, "*");
            }
        }
        for name in out.namespace.clone() {
            write!(out, "{}$", name);
        }
        write!(out, "cxxbridge01${}(", efn.ident);
        for (i, arg) in efn.args.iter().enumerate() {
            if i > 0 {
                write!(out, ", ");
            }
            if types.needs_indirect_abi(&arg.ty) {
                write!(out, "&");
            }
            write!(out, "{}", arg.ident);
            match arg.ty {
                Type::RustBox(_) => write!(out, ".into_raw()"),
                Type::UniquePtr(_) => write!(out, ".release()"),
                _ => {}
            }
        }
        if indirect_return {
            if !efn.args.is_empty() {
                write!(out, ", ");
            }
            write!(out, "reinterpret_cast<");
            write_return_type(out, &efn.ret);
            write!(out, "*>(return$)");
        }
        writeln!(out, ");");
        if indirect_return {
            write!(out, "  return ");
            write_type(out, efn.ret.as_ref().unwrap());
            write!(out, "(*reinterpret_cast<");
            write_return_type(out, &efn.ret);
            writeln!(out, "*>(return$));");
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

fn write_extern_return_type(out: &mut OutFile, ty: &Option<Type>, types: &Types) {
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
        Some(Type::Str(_)) => write!(out, "::cxxbridge::Str::Repr "),
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
        Type::Str(_) => write!(out, "::cxxbridge::Str::Repr "),
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
            Some(Isize) => write!(out, "ssize_t"),
            Some(CxxString) => write!(out, "::std::string"),
            Some(RustString) => write!(out, "::cxxbridge::String"),
            None => write!(out, "{}", ident),
        },
        Type::RustBox(ty) => {
            write!(out, "::cxxbridge::RustBox<");
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
        Type::Str(_) => {
            write!(out, "::cxxbridge::Str");
        }
    }
}

fn write_type_space(out: &mut OutFile, ty: &Type) {
    write_type(out, ty);
    match ty {
        Type::Ident(_) | Type::RustBox(_) | Type::UniquePtr(_) | Type::Str(_) => write!(out, " "),
        Type::Ref(_) => {}
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
                    write_unique_ptr(out, inner);
                }
            }
        }
    }
    out.end_block();

    out.begin_block("namespace cxxbridge01");
    for ty in types {
        if let Type::RustBox(ty) = ty {
            if let Type::Ident(inner) = &ty.inner {
                write_rust_box_impl(out, inner);
            }
        }
    }
    out.end_block();
}

fn write_rust_box_extern(out: &mut OutFile, ident: &Ident) {
    let mut inner = String::new();
    for name in &out.namespace {
        inner += name;
        inner += "::";
    }
    inner += &ident.to_string();
    let instance = inner.replace("::", "$");

    writeln!(out, "#ifndef CXXBRIDGE01_RUST_BOX_{}", instance);
    writeln!(out, "#define CXXBRIDGE01_RUST_BOX_{}", instance);
    writeln!(
        out,
        "void cxxbridge01$rust_box${}$uninit(::cxxbridge::RustBox<{}> *ptr) noexcept;",
        instance, inner,
    );
    writeln!(
        out,
        "void cxxbridge01$rust_box${}$set_raw(::cxxbridge::RustBox<{}> *ptr, {} *raw) noexcept;",
        instance, inner, inner
    );
    writeln!(
        out,
        "void cxxbridge01$rust_box${}$drop(::cxxbridge::RustBox<{}> *ptr) noexcept;",
        instance, inner,
    );
    writeln!(
        out,
        "const {} *cxxbridge01$rust_box${}$deref(const ::cxxbridge::RustBox<{}> *ptr) noexcept;",
        inner, instance, inner,
    );
    writeln!(
        out,
        "{} *cxxbridge01$rust_box${}$deref_mut(::cxxbridge::RustBox<{}> *ptr) noexcept;",
        inner, instance, inner,
    );
    writeln!(out, "#endif // CXXBRIDGE01_RUST_BOX_{}", instance);
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
    writeln!(out, "void RustBox<{}>::uninit() noexcept {{", inner);
    writeln!(
        out,
        "  return cxxbridge01$rust_box${}$uninit(this);",
        instance
    );
    writeln!(out, "}}");

    writeln!(out, "template <>");
    writeln!(
        out,
        "void RustBox<{}>::set_raw({} *raw) noexcept {{",
        inner, inner,
    );
    writeln!(
        out,
        "  return cxxbridge01$rust_box${}$set_raw(this, raw);",
        instance
    );
    writeln!(out, "}}");

    writeln!(out, "template <>");
    writeln!(out, "void RustBox<{}>::drop() noexcept {{", inner);
    writeln!(
        out,
        "  return cxxbridge01$rust_box${}$drop(this);",
        instance
    );
    writeln!(out, "}}");

    writeln!(out, "template <>");
    writeln!(
        out,
        "const {} *RustBox<{}>::deref() const noexcept {{",
        inner, inner,
    );
    writeln!(
        out,
        "  return cxxbridge01$rust_box${}$deref(this);",
        instance
    );
    writeln!(out, "}}");

    writeln!(out, "template <>");
    writeln!(
        out,
        "{} *RustBox<{}>::deref_mut() noexcept {{",
        inner, inner,
    );
    writeln!(
        out,
        "  return cxxbridge01$rust_box${}$deref_mut(this);",
        instance
    );
    writeln!(out, "}}");
}

fn write_unique_ptr(out: &mut OutFile, ident: &Ident) {
    let mut inner = String::new();
    for name in &out.namespace {
        inner += name;
        inner += "::";
    }
    inner += &ident.to_string();
    let instance = inner.replace("::", "$");

    writeln!(out, "#ifndef CXXBRIDGE01_UNIQUE_PTR_{}", instance);
    writeln!(out, "#define CXXBRIDGE01_UNIQUE_PTR_{}", instance);
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
        "void cxxbridge01$unique_ptr${}$null(::std::unique_ptr<{}> *ptr) noexcept {{",
        instance, inner,
    );
    writeln!(out, "  new (ptr) ::std::unique_ptr<{}>();", inner);
    writeln!(out, "}}");
    writeln!(
        out,
        "void cxxbridge01$unique_ptr${}$new(::std::unique_ptr<{}> *ptr, {} *value) noexcept {{",
        instance, inner, inner,
    );
    writeln!(
        out,
        "  new (ptr) ::std::unique_ptr<{}>(new {}(::std::move(*value)));",
        inner, inner,
    );
    writeln!(out, "}}");
    writeln!(
        out,
        "void cxxbridge01$unique_ptr${}$raw(::std::unique_ptr<{}> *ptr, {} *raw) noexcept {{",
        instance, inner, inner,
    );
    writeln!(out, "  new (ptr) ::std::unique_ptr<{}>(raw);", inner);
    writeln!(out, "}}");
    writeln!(
        out,
        "const {} *cxxbridge01$unique_ptr${}$get(const ::std::unique_ptr<{}>& ptr) noexcept {{",
        inner, instance, inner,
    );
    writeln!(out, "  return ptr.get();");
    writeln!(out, "}}");
    writeln!(
        out,
        "{} *cxxbridge01$unique_ptr${}$release(::std::unique_ptr<{}>& ptr) noexcept {{",
        inner, instance, inner,
    );
    writeln!(out, "  return ptr.release();");
    writeln!(out, "}}");
    writeln!(
        out,
        "void cxxbridge01$unique_ptr${}$drop(::std::unique_ptr<{}> *ptr) noexcept {{",
        instance, inner,
    );
    writeln!(out, "  ptr->~unique_ptr();");
    writeln!(out, "}}");
    writeln!(out, "#endif // CXXBRIDGE01_UNIQUE_PTR_{}", instance);
}
