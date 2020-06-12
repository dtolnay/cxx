use crate::syntax::namespace::Namespace;
use crate::syntax::symbol::{self, Symbol};
use crate::syntax::ExternFn;
use proc_macro2::Ident;

const CXXBRIDGE: &str = "cxxbridge03";

macro_rules! join {
    ($($segment:expr),*) => {
        symbol::join(&[$(&$segment),*])
    };
}

pub fn extern_fn(namespace: &Namespace, efn: &ExternFn) -> Symbol {
    let ident = if let Some(alias) = &efn.alias { alias } else { &efn.ident };
    match &efn.receiver {
        Some(receiver) => join!(namespace, CXXBRIDGE, receiver.ty, ident),
        None => join!(namespace, CXXBRIDGE, ident),
    }
}

// The C half of a function pointer trampoline.
pub fn c_trampoline(namespace: &Namespace, efn: &ExternFn, var: &Ident) -> Symbol {
    join!(extern_fn(namespace, efn), var, 0)
}

// The Rust half of a function pointer trampoline.
pub fn r_trampoline(namespace: &Namespace, efn: &ExternFn, var: &Ident) -> Symbol {
    join!(extern_fn(namespace, efn), var, 1)
}
