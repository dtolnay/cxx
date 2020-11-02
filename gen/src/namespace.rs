use crate::syntax::namespace::Namespace;
use crate::syntax::Api;

impl Api {
    pub fn namespace(&self) -> &Namespace {
        match self {
            Api::CxxFunction(efn) | Api::RustFunction(efn) => &efn.ident.cxx.namespace,
            Api::CxxType(ety) | Api::RustType(ety) => &ety.ident.cxx.namespace,
            Api::Enum(enm) => &enm.ident.cxx.namespace,
            Api::Struct(strct) => &strct.ident.cxx.namespace,
            Api::Impl(_) | Api::Include(_) | Api::TypeAlias(_) => Default::default(),
        }
    }
}
