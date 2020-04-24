use crate::syntax::{error, Api};
use proc_macro2::Ident;
use syn::{Error, Result};

pub(crate) fn check(ident: &Ident) -> Result<()> {
    let s = ident.to_string();
    if s.contains("__") {
        Err(Error::new(ident.span(), error::DOUBLE_UNDERSCORE.msg))
    } else if s.starts_with("cxxbridge") {
        Err(Error::new(ident.span(), error::CXXBRIDGE_RESERVED.msg))
    } else {
        Ok(())
    }
}

pub(crate) fn check_all(apis: &[Api], errors: &mut Vec<Error>) {
    for api in apis {
        match api {
            Api::Include(_) => {}
            Api::Struct(strct) => {
                errors.extend(check(&strct.ident).err());
                for field in &strct.fields {
                    errors.extend(check(&field.ident).err());
                }
            }
            Api::Enum(enm) => {
                errors.extend(check(&enm.ident).err());
                for variant in &enm.variants {
                    errors.extend(check(&variant.ident).err());
                }
            }
            Api::CxxType(ety) | Api::RustType(ety) => {
                errors.extend(check(&ety.ident).err());
            }
            Api::CxxFunction(efn) | Api::RustFunction(efn) => {
                errors.extend(check(&efn.ident).err());
                for arg in &efn.args {
                    errors.extend(check(&arg.ident).err());
                }
            }
        }
    }
}
