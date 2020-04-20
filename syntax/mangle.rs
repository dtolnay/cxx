use crate::syntax::namespace::Namespace;
use crate::syntax::{symbol, ExternFn};

const CXXBRIDGE: &str = "cxxbridge02";

macro_rules! join {
    ($($segment:expr),*) => {
        symbol::join(&[$(&$segment),*])
    };
}

pub fn extern_fn(namespace: &Namespace, efn: &ExternFn) -> String {
    match &efn.receiver {
        Some(receiver) => join!(namespace, CXXBRIDGE, receiver.ident, efn.ident),
        None => join!(namespace, CXXBRIDGE, efn.ident),
    }
    .to_string()
}
