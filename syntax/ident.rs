use crate::syntax::check::Check;
use crate::syntax::{error, Api, Pair};
use proc_macro2::Ident;

fn check(cx: &mut Check, name: &Pair) {
    for segment in &name.namespace {
        check_cxx_ident(cx, segment);
    }
    check_cxx_ident(cx, &name.cxx);
    check_rust_ident(cx, &name.rust);

    fn check_cxx_ident(cx: &mut Check, ident: &Ident) {
        let s = ident.to_string();
        if s.starts_with("cxxbridge") {
            cx.error(ident, error::CXXBRIDGE_RESERVED.msg);
        }
        if s.contains("__") {
            cx.error(ident, error::DOUBLE_UNDERSCORE.msg);
        }
    }

    fn check_rust_ident(cx: &mut Check, ident: &Ident) {
        let s = ident.to_string();
        if s.starts_with("cxxbridge") {
            cx.error(ident, error::CXXBRIDGE_RESERVED.msg);
        }
    }
}

pub(crate) fn check_all(cx: &mut Check, apis: &[Api]) {
    for api in apis {
        match api {
            Api::Include(_) | Api::Impl(_) => {}
            Api::Struct(strct) => {
                check(cx, &strct.name);
                for field in &strct.fields {
                    check(cx, &field.name);
                }
            }
            Api::Enum(enm) => {
                check(cx, &enm.name);
                for variant in &enm.variants {
                    check(cx, &variant.name);
                }
            }
            Api::CxxType(ety) | Api::RustType(ety) => {
                check(cx, &ety.name);
            }
            Api::CxxFunction(efn) | Api::RustFunction(efn) => {
                check(cx, &efn.name);
                for arg in &efn.args {
                    check(cx, &arg.name);
                }
            }
            Api::TypeAlias(alias) => {
                check(cx, &alias.name);
            }
        }
    }
}
