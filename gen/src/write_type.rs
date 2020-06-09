use crate::gen::out::OutFile;
use crate::syntax::atom::Atom::{self, *};
use crate::syntax::namespace::Namespace;
use crate::syntax::symbol::Symbol;
use crate::syntax::{mangle, ExternFn, Signature, Type, Types};
use proc_macro2::Ident;

pub(crate) trait WriteType {
    fn needs_rust_string(&self) -> bool;
    fn needs_rust_str(&self) -> bool;
    fn needs_rust_slice(&self) -> bool;
    fn needs_rust_box(&self) -> bool;
    fn needs_rust_vec(&self) -> bool;
    fn needs_rust_fn(&self) -> bool;
    fn needs_rust_isize(&self) -> bool;

    fn include_type_traits(&self) -> bool;
    fn include_array(&self) -> bool;
    fn include_cstdint(&self) -> bool;
    fn include_string(&self) -> bool;
    fn include_base_tsd(&self) -> bool;

    fn write_extern_arg_cxx(&self, out: &mut OutFile, types: &Types, ident: &Ident);
    fn write_extern_arg_rust(&self, out: &mut OutFile, types: &Types, ident: &Ident);
    fn write_space_after_type(&self, out: &mut OutFile);
    fn write_indirect_return_type(&self, out: &mut OutFile);
    fn write_indirect_return_type_space(&self, out: &mut OutFile);
    fn write_return_args(&self, out: &mut OutFile, types: &Types, ident: &Ident);
    fn write_type(&self, out: &mut OutFile);
    fn write_type_space(&self, out: &mut OutFile);
    fn write_function_pointer_trampoline(
        &self,
        out: &mut OutFile,
        efn: &ExternFn,
        var: &Ident,
        types: &Types,
    );
    fn write_extern(&self, out: &mut OutFile, types: &Types);
    fn write_impl(&self, out: &mut OutFile);

    fn write_rust_shim_return_prefix(&self, out: &mut OutFile);
    fn write_rust_shim_return_arg(&self, out: &mut OutFile, types: &Types, ident: &Ident);
    fn write_rust_shim_return_suffix(&self, out: &mut OutFile);

    fn to_typename(&self, namespace: &Namespace) -> String;
    fn to_mangled(&self, namespace: &Namespace) -> String;
}

pub(crate) trait WriteOptionType {
    fn write_return_type(&self, out: &mut OutFile);
    fn write_extern_return_type_space(&self, out: &mut OutFile, types: &Types);
    fn write_return_prefix(&self, out: &mut OutFile, indirect_return: bool);
    fn write_return_suffix(&self, out: &mut OutFile, indirect_return: bool);
}

impl WriteOptionType for Option<Type> {
    fn write_return_type(&self, out: &mut OutFile) {
        match self {
            None => write!(out, "void "),
            Some(ty) => ty.write_type_space(out),
        }
    }

    fn write_extern_return_type_space(&self, out: &mut OutFile, types: &Types) {
        match self {
            Some(Type::RustBox(ty)) | Some(Type::UniquePtr(ty)) => {
                ty.inner.write_type_space(out);
                write!(out, "*");
            }
            Some(Type::Ref(ty)) => {
                if ty.mutability.is_none() {
                    write!(out, "const ");
                }
                ty.inner.write_type(out);
                write!(out, " *");
            }
            Some(Type::Str(_)) => write!(out, "::rust::Str::Repr "),
            Some(Type::SliceRefU8(_)) => write!(out, "::rust::Slice<uint8_t>::Repr "),
            Some(ty) if types.needs_indirect_abi(ty) => write!(out, "void "),
            _ => self.write_return_type(out),
        }
    }

    fn write_return_prefix(&self, out: &mut OutFile, indirect_return: bool) {
        match self {
            Some(Type::Ref(_)) => write!(out, "&"),
            Some(Type::Str(_)) if !indirect_return => write!(out, "::rust::Str::Repr("),
            Some(Type::SliceRefU8(_)) if !indirect_return => {
                write!(out, "::rust::Slice<uint8_t>::Repr(")
            }
            _ => {}
        }
    }

    fn write_return_suffix(&self, out: &mut OutFile, indirect_return: bool) {
        match self {
            Some(Type::RustBox(_)) => write!(out, ".into_raw()"),
            Some(Type::UniquePtr(_)) => write!(out, ".release()"),
            Some(Type::Str(_)) | Some(Type::SliceRefU8(_)) if !indirect_return => write!(out, ")"),
            _ => {}
        }
    }
}

impl WriteType for Type {
    fn needs_rust_string(&self) -> bool {
        self == RustString
    }

    fn needs_rust_str(&self) -> bool {
        match self {
            Type::Str(_) => true,
            _ => false,
        }
    }

    fn needs_rust_slice(&self) -> bool {
        match self {
            Type::Slice(_) => true,
            Type::SliceRefU8(_) => true,
            _ => false,
        }
    }

    fn needs_rust_box(&self) -> bool {
        match self {
            Type::RustBox(_) => true,
            _ => false,
        }
    }

    fn needs_rust_vec(&self) -> bool {
        match self {
            Type::RustVec(_) => true,
            _ => false,
        }
    }

    fn needs_rust_fn(&self) -> bool {
        match self {
            Type::Fn(_) => true,
            _ => false,
        }
    }

    fn needs_rust_isize(&self) -> bool {
        self == Isize
    }

    fn include_type_traits(&self) -> bool {
        match self {
            Type::RustBox(_) => true,
            Type::RustVec(_) => true,
            _ => false,
        }
    }

    fn include_array(&self) -> bool {
        match self {
            Type::RustVec(_) => true,
            ty if ty == RustString => true,
            _ => false,
        }
    }

    fn include_cstdint(&self) -> bool {
        match self {
            Type::Str(_) => true,
            ty if ty == RustString => true,
            _ => false,
        }
    }

    fn include_string(&self) -> bool {
        match self {
            Type::Str(_) => true,
            ty if ty == RustString => true,
            _ => false,
        }
    }

    fn include_base_tsd(&self) -> bool {
        self == Isize
    }

    fn write_extern_arg_cxx(&self, out: &mut OutFile, types: &Types, ident: &Ident) {
        if self == RustString {
            write!(out, "const ");
        } else if let Type::RustVec(_) = self {
            write!(out, "const ");
        }
        self.write_extern_arg_rust(out, types, ident);
    }

    fn write_extern_arg_rust(&self, out: &mut OutFile, types: &Types, ident: &Ident) {
        match &self {
            Type::RustBox(ty) | Type::UniquePtr(ty) | Type::CxxVector(ty) => {
                ty.inner.write_type_space(out);
                write!(out, "*");
            }
            Type::Str(_) => write!(out, "::rust::Str::Repr "),
            Type::SliceRefU8(_) => write!(out, "::rust::Slice<uint8_t>::Repr "),
            _ => self.write_type_space(out),
        }
        if types.needs_indirect_abi(&self) {
            write!(out, "*");
        }
        write!(out, "{}", ident);
    }

    fn write_space_after_type(&self, out: &mut OutFile) {
        match self {
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

    fn write_indirect_return_type(&self, out: &mut OutFile) {
        match self {
            Type::RustBox(ty) | Type::UniquePtr(ty) => {
                ty.inner.write_type_space(out);
                write!(out, "*");
            }
            Type::Ref(ty) => {
                if ty.mutability.is_none() {
                    write!(out, "const ");
                }
                ty.inner.write_type(out);
                write!(out, " *");
            }
            Type::Str(_) => write!(out, "::rust::Str::Repr"),
            Type::SliceRefU8(_) => write!(out, "::rust::Slice<uint8_t>::Repr"),
            _ => self.write_type(out),
        }
    }

    fn write_indirect_return_type_space(&self, out: &mut OutFile) {
        self.write_indirect_return_type(out);
        match self {
            Type::RustBox(_) | Type::UniquePtr(_) | Type::Ref(_) => {}
            Type::Str(_) | Type::SliceRefU8(_) => write!(out, " "),
            _ => self.write_space_after_type(out),
        }
    }

    fn write_return_args(&self, out: &mut OutFile, types: &Types, ident: &Ident) {
        if let Type::RustBox(_) = self {
            self.write_type(out);
            write!(out, "::from_raw({})", ident);
        } else if let Type::UniquePtr(_) = self {
            self.write_type(out);
            write!(out, "({})", ident);
        } else if self == RustString {
            write!(out, "::rust::String(::rust::unsafe_bitcopy, *{})", ident);
        } else if let Type::RustVec(_) = self {
            self.write_type(out);
            write!(out, "(::rust::unsafe_bitcopy, *{})", ident);
        } else if types.needs_indirect_abi(self) {
            out.include.utility = true;
            write!(out, "::std::move(*{})", ident);
        } else {
            write!(out, "{}", ident);
        }
    }

    fn write_type(&self, out: &mut OutFile) {
        match self {
            Type::Ident(ident) => match Atom::from(ident) {
                Some(atom) => write_atom(out, atom),
                None => write!(out, "{}", ident),
            },
            Type::RustBox(ty) => {
                write!(out, "::rust::Box<");
                ty.inner.write_type(out);
                write!(out, ">");
            }
            Type::RustVec(ty) => {
                write!(out, "::rust::Vec<");
                ty.inner.write_type(out);
                write!(out, ">");
            }
            Type::UniquePtr(ptr) => {
                write!(out, "::std::unique_ptr<");
                ptr.inner.write_type(out);
                write!(out, ">");
            }
            Type::CxxVector(ty) => {
                write!(out, "::std::vector<");
                ty.inner.write_type(out);
                write!(out, ">");
            }
            Type::Ref(r) => {
                if r.mutability.is_none() {
                    write!(out, "const ");
                }
                r.inner.write_type(out);
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
                    Some(ret) => ret.write_type(out),
                    None => write!(out, "void"),
                }
                write!(out, "(");
                for (i, arg) in f.args.iter().enumerate() {
                    if i > 0 {
                        write!(out, ", ");
                    }
                    arg.ty.write_type(out);
                }
                write!(out, ")>");
            }
            Type::Void(_) => unreachable!(),
        }
    }

    fn write_type_space(&self, out: &mut OutFile) {
        self.write_type(out);
        self.write_space_after_type(out);
    }

    fn write_function_pointer_trampoline(
        &self,
        out: &mut OutFile,
        efn: &ExternFn,
        var: &Ident,
        types: &Types,
    ) {
        if let Type::Fn(f) = &self {
            out.next_section();
            let r_trampoline = mangle::r_trampoline(&out.namespace, efn, var);
            let indirect_call = true;
            write_rust_function_decl_impl(out, &r_trampoline, f, types, indirect_call);

            out.next_section();
            let c_trampoline = mangle::c_trampoline(&out.namespace, efn, var).to_string();
            write_rust_function_shim_impl(
                out,
                &c_trampoline,
                f,
                types,
                &r_trampoline,
                indirect_call,
            );
        }
    }

    fn write_extern(&self, out: &mut OutFile, types: &Types) {
        if let Type::RustBox(ty) = self {
            if let Type::Ident(inner) = &ty.inner {
                out.next_section();
                write_rust_box_extern(out, inner);
            }
        } else if let Type::RustVec(ty) = self {
            if let Type::Ident(inner) = &ty.inner {
                if Atom::from(inner).is_none() {
                    out.next_section();
                    write_rust_vec_extern(out, inner);
                }
            }
        } else if let Type::UniquePtr(ptr) = self {
            if let Type::Ident(inner) = &ptr.inner {
                if Atom::from(inner).is_none() && !types.aliases.contains_key(inner) {
                    out.next_section();
                    write_unique_ptr(out, inner, types);
                }
            }
        } else if let Type::CxxVector(ptr) = self {
            if let Type::Ident(inner) = &ptr.inner {
                if Atom::from(inner).is_none() && !types.aliases.contains_key(inner) {
                    out.next_section();
                    write_cxx_vector(out, self, inner, types);
                }
            }
        }
    }

    fn write_impl(&self, out: &mut OutFile) {
        if let Type::RustBox(ty) = self {
            if let Type::Ident(inner) = &ty.inner {
                write_rust_box_impl(out, inner);
            }
        } else if let Type::RustVec(ty) = self {
            if let Type::Ident(inner) = &ty.inner {
                if Atom::from(inner).is_none() {
                    write_rust_vec_impl(out, inner);
                }
            }
        }
    }

    fn write_rust_shim_return_prefix(&self, out: &mut OutFile) {
        match self {
            Type::RustBox(_) => {
                self.write_type(out);
                write!(out, "::from_raw(");
            }
            Type::UniquePtr(_) => {
                self.write_type(out);
                write!(out, "(");
            }
            Type::Ref(_) => write!(out, "*"),
            _ => {}
        }
    }

    fn write_rust_shim_return_arg(&self, out: &mut OutFile, types: &Types, ident: &Ident) {
        match self {
            Type::Str(_) => write!(out, "::rust::Str::Repr("),
            Type::SliceRefU8(_) => write!(out, "::rust::Slice<uint8_t>::Repr("),
            ty if types.needs_indirect_abi(ty) => write!(out, "&"),
            _ => {}
        }
        write!(out, "{}", ident);
        match self {
            Type::RustBox(_) => write!(out, ".into_raw()"),
            Type::UniquePtr(_) => write!(out, ".release()"),
            Type::Str(_) | Type::SliceRefU8(_) => write!(out, ")"),
            ty if ty != RustString && types.needs_indirect_abi(ty) => write!(out, "$.value"),
            _ => {}
        }
    }

    fn write_rust_shim_return_suffix(&self, out: &mut OutFile) {
        if let Type::RustBox(_) | Type::UniquePtr(_) = self {
            write!(out, ")");
        }
    }

    // Only called for legal referent types of unique_ptr and element types of
    // std::vector and Vec.
    fn to_typename(&self, namespace: &Namespace) -> String {
        match self {
            Type::Ident(ident) => {
                let mut path = String::new();
                for name in namespace {
                    path += &name.to_string();
                    path += "::";
                }
                path += &ident.to_string();
                path
            }
            Type::CxxVector(ptr) => format!("::std::vector<{}>", ptr.inner.to_typename(namespace)),
            _ => unreachable!(),
        }
    }

    // Only called for legal referent types of unique_ptr and element types of
    // std::vector and Vec.
    fn to_mangled(&self, namespace: &Namespace) -> String {
        match self {
            Type::Ident(_) => self.to_typename(namespace).replace("::", "$"),
            Type::CxxVector(ptr) => format!("std$vector${}", ptr.inner.to_mangled(namespace)),
            _ => unreachable!(),
        }
    }
}

pub(crate) fn write_atom(out: &mut OutFile, atom: Atom) {
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

pub(crate) fn write_rust_function_decl_impl(
    out: &mut OutFile,
    link_name: &Symbol,
    sig: &Signature,
    types: &Types,
    indirect_call: bool,
) {
    if sig.throws {
        write!(out, "::rust::Str::Repr ");
    } else {
        sig.ret.write_extern_return_type_space(out, types);
    }
    write!(out, "{}(", link_name);
    let mut needs_comma = false;
    if let Some(receiver) = &sig.receiver {
        if receiver.mutability.is_none() {
            write!(out, "const ");
        }
        write!(out, "{} &self", receiver.ty);
        needs_comma = true;
    }
    for arg in &sig.args {
        if needs_comma {
            write!(out, ", ");
        }
        arg.ty.write_extern_arg_rust(out, types, &arg.ident);
        needs_comma = true;
    }
    if indirect_return(sig, types) {
        if needs_comma {
            write!(out, ", ");
        }
        sig.ret.write_return_type(out);
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

pub(crate) fn write_rust_function_shim_decl(
    out: &mut OutFile,
    local_name: &str,
    sig: &Signature,
    indirect_call: bool,
) {
    sig.ret.write_return_type(out);
    write!(out, "{}(", local_name);
    for (i, arg) in sig.args.iter().enumerate() {
        if i > 0 {
            write!(out, ", ");
        }
        arg.ty.write_type_space(out);
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

pub(crate) fn write_rust_function_shim_impl(
    out: &mut OutFile,
    local_name: &str,
    sig: &Signature,
    types: &Types,
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
        if arg.ty != RustString && types.needs_indirect_abi(&arg.ty) {
            out.include.utility = true;
            write!(out, "  ::rust::ManuallyDrop<");
            arg.ty.write_type(out);
            writeln!(out, "> {}$(::std::move({0}));", arg.ident);
        }
    }
    write!(out, "  ");
    let indirect_return = indirect_return(sig, types);
    if indirect_return {
        write!(out, "::rust::MaybeUninit<");
        sig.ret.as_ref().unwrap().write_type(out);
        writeln!(out, "> return$;");
        write!(out, "  ");
    } else if let Some(ret) = &sig.ret {
        write!(out, "return ");
        ret.write_rust_shim_return_prefix(out);
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
        arg.ty.write_rust_shim_return_arg(out, types, &arg.ident);
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
        ret.write_rust_shim_return_suffix(out);
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

pub(crate) fn indirect_return(sig: &Signature, types: &Types) -> bool {
    sig.ret
        .as_ref()
        .map_or(false, |ret| sig.throws || types.needs_indirect_abi(ret))
}

fn write_rust_box_extern(out: &mut OutFile, ident: &Ident) {
    let mut inner = String::new();
    for name in &out.namespace {
        inner += &name.to_string();
        inner += "::";
    }
    inner += &ident.to_string();
    let instance = inner.replace("::", "$");

    writeln!(out, "#ifndef CXXBRIDGE03_RUST_BOX_{}", instance);
    writeln!(out, "#define CXXBRIDGE03_RUST_BOX_{}", instance);
    writeln!(
        out,
        "void cxxbridge03$box${}$uninit(::rust::Box<{}> *ptr) noexcept;",
        instance, inner,
    );
    writeln!(
        out,
        "void cxxbridge03$box${}$drop(::rust::Box<{}> *ptr) noexcept;",
        instance, inner,
    );
    writeln!(out, "#endif // CXXBRIDGE03_RUST_BOX_{}", instance);
}

fn write_rust_vec_extern(out: &mut OutFile, element: &Ident) {
    let element = Type::Ident(element.clone());
    let inner = element.to_typename(&out.namespace);
    let instance = element.to_mangled(&out.namespace);

    writeln!(out, "#ifndef CXXBRIDGE03_RUST_VEC_{}", instance);
    writeln!(out, "#define CXXBRIDGE03_RUST_VEC_{}", instance);
    writeln!(
        out,
        "void cxxbridge03$rust_vec${}$new(const ::rust::Vec<{}> *ptr) noexcept;",
        instance, inner,
    );
    writeln!(
        out,
        "void cxxbridge03$rust_vec${}$drop(::rust::Vec<{}> *ptr) noexcept;",
        instance, inner,
    );
    writeln!(
        out,
        "size_t cxxbridge03$rust_vec${}$len(const ::rust::Vec<{}> *ptr) noexcept;",
        instance, inner,
    );
    writeln!(
        out,
        "const {} *cxxbridge03$rust_vec${}$data(const ::rust::Vec<{0}> *ptr) noexcept;",
        inner, instance,
    );
    writeln!(
        out,
        "size_t cxxbridge03$rust_vec${}$stride() noexcept;",
        instance,
    );
    writeln!(out, "#endif // CXXBRIDGE03_RUST_VEC_{}", instance);
}

fn write_rust_box_impl(out: &mut OutFile, ident: &Ident) {
    let mut inner = String::new();
    for name in &out.namespace {
        inner += &name.to_string();
        inner += "::";
    }
    inner += &ident.to_string();
    let instance = inner.replace("::", "$");

    writeln!(out, "template <>");
    writeln!(out, "void Box<{}>::uninit() noexcept {{", inner);
    writeln!(out, "  cxxbridge03$box${}$uninit(this);", instance);
    writeln!(out, "}}");

    writeln!(out, "template <>");
    writeln!(out, "void Box<{}>::drop() noexcept {{", inner);
    writeln!(out, "  cxxbridge03$box${}$drop(this);", instance);
    writeln!(out, "}}");
}

fn write_rust_vec_impl(out: &mut OutFile, element: &Ident) {
    let element = Type::Ident(element.clone());
    let inner = element.to_typename(&out.namespace);
    let instance = element.to_mangled(&out.namespace);

    writeln!(out, "template <>");
    writeln!(out, "Vec<{}>::Vec() noexcept {{", inner);
    writeln!(out, "  cxxbridge03$rust_vec${}$new(this);", instance);
    writeln!(out, "}}");

    writeln!(out, "template <>");
    writeln!(out, "void Vec<{}>::drop() noexcept {{", inner);
    writeln!(
        out,
        "  return cxxbridge03$rust_vec${}$drop(this);",
        instance,
    );
    writeln!(out, "}}");

    writeln!(out, "template <>");
    writeln!(out, "size_t Vec<{}>::size() const noexcept {{", inner);
    writeln!(out, "  return cxxbridge03$rust_vec${}$len(this);", instance);
    writeln!(out, "}}");

    writeln!(out, "template <>");
    writeln!(out, "const {} *Vec<{0}>::data() const noexcept {{", inner);
    writeln!(
        out,
        "  return cxxbridge03$rust_vec${}$data(this);",
        instance,
    );
    writeln!(out, "}}");

    writeln!(out, "template <>");
    writeln!(out, "size_t Vec<{}>::stride() noexcept {{", inner);
    writeln!(out, "  return cxxbridge03$rust_vec${}$stride();", instance);
    writeln!(out, "}}");
}

fn write_unique_ptr(out: &mut OutFile, ident: &Ident, types: &Types) {
    let ty = Type::Ident(ident.clone());
    let instance = ty.to_mangled(&out.namespace);

    writeln!(out, "#ifndef CXXBRIDGE03_UNIQUE_PTR_{}", instance);
    writeln!(out, "#define CXXBRIDGE03_UNIQUE_PTR_{}", instance);

    write_unique_ptr_common(out, &ty, types);

    writeln!(out, "#endif // CXXBRIDGE03_UNIQUE_PTR_{}", instance);
}

// Shared by UniquePtr<T> and UniquePtr<CxxVector<T>>.
fn write_unique_ptr_common(out: &mut OutFile, ty: &Type, types: &Types) {
    out.include.utility = true;
    let inner = ty.to_typename(&out.namespace);
    let instance = ty.to_mangled(&out.namespace);

    let can_construct_from_value = match ty {
        Type::Ident(ident) => types.structs.contains_key(ident),
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
        "void cxxbridge03$unique_ptr${}$null(::std::unique_ptr<{}> *ptr) noexcept {{",
        instance, inner,
    );
    writeln!(out, "  new (ptr) ::std::unique_ptr<{}>();", inner);
    writeln!(out, "}}");
    if can_construct_from_value {
        writeln!(
            out,
            "void cxxbridge03$unique_ptr${}$new(::std::unique_ptr<{}> *ptr, {} *value) noexcept {{",
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
        "void cxxbridge03$unique_ptr${}$raw(::std::unique_ptr<{}> *ptr, {} *raw) noexcept {{",
        instance, inner, inner,
    );
    writeln!(out, "  new (ptr) ::std::unique_ptr<{}>(raw);", inner);
    writeln!(out, "}}");
    writeln!(
        out,
        "const {} *cxxbridge03$unique_ptr${}$get(const ::std::unique_ptr<{}>& ptr) noexcept {{",
        inner, instance, inner,
    );
    writeln!(out, "  return ptr.get();");
    writeln!(out, "}}");
    writeln!(
        out,
        "{} *cxxbridge03$unique_ptr${}$release(::std::unique_ptr<{}>& ptr) noexcept {{",
        inner, instance, inner,
    );
    writeln!(out, "  return ptr.release();");
    writeln!(out, "}}");
    writeln!(
        out,
        "void cxxbridge03$unique_ptr${}$drop(::std::unique_ptr<{}> *ptr) noexcept {{",
        instance, inner,
    );
    writeln!(out, "  ptr->~unique_ptr();");
    writeln!(out, "}}");
}

fn write_cxx_vector(out: &mut OutFile, vector_ty: &Type, element: &Ident, types: &Types) {
    let element = Type::Ident(element.clone());
    let inner = element.to_typename(&out.namespace);
    let instance = element.to_mangled(&out.namespace);

    writeln!(out, "#ifndef CXXBRIDGE03_VECTOR_{}", instance);
    writeln!(out, "#define CXXBRIDGE03_VECTOR_{}", instance);
    writeln!(
        out,
        "size_t cxxbridge03$std$vector${}$size(const ::std::vector<{}> &s) noexcept {{",
        instance, inner,
    );
    writeln!(out, "  return s.size();");
    writeln!(out, "}}");
    writeln!(
        out,
        "const {} *cxxbridge03$std$vector${}$get_unchecked(const ::std::vector<{}> &s, size_t pos) noexcept {{",
        inner, instance, inner,
    );
    writeln!(out, "  return &s[pos];");
    writeln!(out, "}}");

    write_unique_ptr_common(out, vector_ty, types);

    writeln!(out, "#endif // CXXBRIDGE03_VECTOR_{}", instance);
}
