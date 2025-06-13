use syntax::namespace::Namespace;
use syntax::Api;

pub fn namespace(api: &Api) -> &Namespace {
    match api {
        Api::CxxFunction(efn) | Api::RustFunction(efn) => &efn.name.namespace,
        Api::CxxType(ety) | Api::RustType(ety) => &ety.name.namespace,
        Api::Enum(enm) => &enm.name.namespace,
        Api::Struct(strct) => &strct.name.namespace,
        Api::Impl(_) | Api::Include(_) | Api::TypeAlias(_) => Default::default(),
    }
}