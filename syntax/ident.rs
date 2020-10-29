use crate::syntax::check::Check;
use crate::syntax::{error, Api, CppName};
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

fn check_ident(cx: &mut Check, ident: &CppName) {
    for segment in &ident.ns {
        check(cx, segment);
    }
    check(cx, &ident.ident);
}

pub(crate) fn check_all(cx: &mut Check, apis: &[Api]) {
    for api in apis {
        match api {
            Api::Include(_) | Api::Impl(_) => {}
            Api::Struct(strct) => {
                check_ident(cx, &strct.ident.cxx);
                for field in &strct.fields {
                    check(cx, &field.ident);
                }
            }
            Api::Enum(enm) => {
                check_ident(cx, &enm.ident.cxx);
                for variant in &enm.variants {
                    check(cx, &variant.ident);
                }
            }
            Api::CxxType(ety) | Api::RustType(ety) => {
                check_ident(cx, &ety.ident.cxx);
            }
            Api::CxxFunction(efn) | Api::RustFunction(efn) => {
                check(cx, &efn.ident.rust);
                for arg in &efn.args {
                    check(cx, &arg.ident);
                }
            }
            Api::TypeAlias(alias) => {
                check_ident(cx, &alias.ident.cxx);
            }
        }
    }
}
