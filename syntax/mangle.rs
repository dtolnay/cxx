use crate::syntax::symbol::{self, Symbol};
use crate::syntax::{ExternFn, Pair, Types};
use proc_macro2::Ident;

const CXXBRIDGE: &str = "cxxbridge1";

macro_rules! join {
    ($($segment:expr),+ $(,)?) => {
        symbol::join(&[$(&$segment),+])
    };
}

pub fn extern_fn(efn: &ExternFn, types: &Types) -> Symbol {
    match &efn.receiver {
        Some(receiver) => {
            let receiver_ident = types.resolve(&receiver.ty);
            join!(
                efn.name.namespace,
                CXXBRIDGE,
                receiver_ident.cxx,
                efn.name.rust,
            )
        }
        None => join!(efn.name.namespace, CXXBRIDGE, efn.name.rust),
    }
}

pub fn operator(receiver: &Pair, operator: &'static str) -> Symbol {
    join!(
        receiver.namespace,
        CXXBRIDGE,
        receiver.cxx,
        "operator",
        operator,
    )
}

// The C half of a function pointer trampoline.
pub fn c_trampoline(efn: &ExternFn, var: &Ident, types: &Types) -> Symbol {
    join!(extern_fn(efn, types), var, 0)
}

// The Rust half of a function pointer trampoline.
pub fn r_trampoline(efn: &ExternFn, var: &Ident, types: &Types) -> Symbol {
    join!(extern_fn(efn, types), var, 1)
}
