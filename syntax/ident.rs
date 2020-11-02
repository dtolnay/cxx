use crate::syntax::check::Check;
use crate::syntax::{error, Api, Pair};
use proc_macro2::Ident;

fn check(cx: &mut Check, ident: &Ident) {
    let s = ident.to_string();
    if s.starts_with("cxxbridge") {
        cx.error(ident, error::CXXBRIDGE_RESERVED.msg);
    }
    if s.contains("__") {
        cx.error(ident, error::DOUBLE_UNDERSCORE.msg);
    }
}

fn check_ident(cx: &mut Check, name: &Pair) {
    for segment in &name.namespace {
        check(cx, segment);
    }
    check(cx, &name.cxx);
}

pub(crate) fn check_all(cx: &mut Check, apis: &[Api]) {
    for api in apis {
        match api {
            Api::Include(_) | Api::Impl(_) => {}
            Api::Struct(strct) => {
                check_ident(cx, &strct.name);
                for field in &strct.fields {
                    check(cx, &field.ident);
                }
            }
            Api::Enum(enm) => {
                check_ident(cx, &enm.name);
                for variant in &enm.variants {
                    check(cx, &variant.ident);
                }
            }
            Api::CxxType(ety) | Api::RustType(ety) => {
                check_ident(cx, &ety.name);
            }
            Api::CxxFunction(efn) | Api::RustFunction(efn) => {
                check(cx, &efn.name.rust);
                for arg in &efn.args {
                    check(cx, &arg.ident);
                }
            }
            Api::TypeAlias(alias) => {
                check_ident(cx, &alias.name);
            }
        }
    }
}
