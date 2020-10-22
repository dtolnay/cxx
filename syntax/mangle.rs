use crate::syntax::symbol::{self, Symbol};
use crate::syntax::ExternFn;
use proc_macro2::Ident;

const CXXBRIDGE: &str = "cxxbridge05";

macro_rules! join {
    ($($segment:expr),*) => {
        symbol::join(&[$(&$segment),*])
    };
}

pub fn extern_fn(efn: &ExternFn) -> Symbol {
    match &efn.receiver {
        Some(receiver) => join!(efn.ident.cxx.ns, CXXBRIDGE, receiver.ty, efn.ident.rust),
        None => join!(efn.ident.cxx.ns, CXXBRIDGE, efn.ident.rust),
    }
}

// The C half of a function pointer trampoline.
pub fn c_trampoline(efn: &ExternFn, var: &Ident) -> Symbol {
    join!(extern_fn(efn), var, 0)
}

// The Rust half of a function pointer trampoline.
pub fn r_trampoline(efn: &ExternFn, var: &Ident) -> Symbol {
    join!(extern_fn(efn), var, 1)
}
