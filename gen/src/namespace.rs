use crate::syntax::namespace::Namespace;
use crate::syntax::Api;

impl Api {
    pub fn namespace(&self) -> &Namespace {
        match self {
            Api::CxxFunction(efn) | Api::RustFunction(efn) => &efn.ident.namespace,
            Api::CxxType(ety) | Api::RustType(ety) => &ety.ident.namespace,
            Api::Enum(enm) => &enm.ident.namespace,
            Api::Struct(strct) => &strct.ident.namespace,
            Api::Impl(_) | Api::Include(_) | Api::TypeAlias(_) => Default::default(),
        }
    }
}
