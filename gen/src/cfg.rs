use crate::gen::{CfgEvaluator, CfgResult};
use crate::syntax::cfg::CfgExpr;
use crate::syntax::report::Errors;
use crate::syntax::Api;
use quote::quote;
use std::collections::BTreeSet as Set;

pub(super) struct UnsupportedCfgEvaluator;

impl CfgEvaluator for UnsupportedCfgEvaluator {
    fn eval(&self, name: &str, value: Option<&str>) -> CfgResult {
        let _ = name;
        let _ = value;
        let msg = "cfg attribute is not supported".to_owned();
        CfgResult::Undetermined { msg }
    }
}

pub(super) fn strip(cx: &mut Errors, cfg_evaluator: &dyn CfgEvaluator, apis: &mut Vec<Api>) {
    let mut already_errors = Set::new();
    apis.retain(|api| eval(cx, &mut already_errors, cfg_evaluator, api.cfg()));
    for api in apis {
        match api {
            Api::Struct(strct) => strct
                .fields
                .retain(|field| eval(cx, &mut already_errors, cfg_evaluator, &field.cfg)),
            Api::Enum(enm) => enm
                .variants
                .retain(|variant| eval(cx, &mut already_errors, cfg_evaluator, &variant.cfg)),
            _ => {}
        }
    }
}

fn eval(
    cx: &mut Errors,
    already_errors: &mut Set<String>,
    cfg_evaluator: &dyn CfgEvaluator,
    expr: &CfgExpr,
) -> bool {
    match expr {
        CfgExpr::Unconditional => true,
        CfgExpr::Eq(ident, string) => {
            let key = ident.to_string();
            let value = string.as_ref().map(|string| string.value());
            match cfg_evaluator.eval(&key, value.as_deref()) {
                CfgResult::True => true,
                CfgResult::False => false,
                CfgResult::Undetermined { msg } => {
                    if already_errors.insert(msg.clone()) {
                        let span = quote!(#ident #string);
                        cx.error(span, msg);
                    }
                    false
                }
            }
        }
        CfgExpr::All(list) => list
            .iter()
            .all(|expr| eval(cx, already_errors, cfg_evaluator, expr)),
        CfgExpr::Any(list) => list
            .iter()
            .any(|expr| eval(cx, already_errors, cfg_evaluator, expr)),
        CfgExpr::Not(expr) => !eval(cx, already_errors, cfg_evaluator, expr),
    }
}

impl Api {
    fn cfg(&self) -> &CfgExpr {
        match self {
            Api::Include(include) => &include.cfg,
            Api::Struct(strct) => &strct.cfg,
            Api::Enum(enm) => &enm.cfg,
            Api::CxxType(ety) | Api::RustType(ety) => &ety.cfg,
            Api::CxxFunction(efn) | Api::RustFunction(efn) => &efn.cfg,
            Api::TypeAlias(alias) => &alias.cfg,
            Api::Impl(imp) => &imp.cfg,
        }
    }
}
