use crate::syntax::check::Check;
use crate::syntax::namespace::Namespace;
use crate::syntax::{error, Api};
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

pub(crate) fn check_all(cx: &mut Check, namespace: &Namespace, apis: &[Api]) {
    for segment in namespace {
        check(cx, segment);
    }

    for api in apis {
        match api {
            Api::Include(_) => {}
            Api::Struct(strct) => {
                check(cx, &strct.ident);
                for field in &strct.fields {
                    check(cx, &field.ident);
                }
            }
            Api::Enum(enm) => {
                check(cx, &enm.ident);
                for variant in &enm.variants {
                    check(cx, &variant.ident);
                }
            }
            Api::CxxType(ety) | Api::RustType(ety) => {
                check(cx, &ety.ident);
            }
            Api::CxxFunction(efn) | Api::RustFunction(efn) => {
                check(cx, &efn.ident);
                for arg in &efn.args {
                    check(cx, &arg.ident);
                }
            }
            Api::TypeAlias(alias) => {
                check(cx, &alias.ident);
            }
        }
    }
}
