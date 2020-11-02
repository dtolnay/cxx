use crate::syntax::{Api, Types};
use std::iter::FromIterator;

pub fn sort<'a>(apis: &'a [Api], types: &Types) -> Vec<&'a Api> {
    // TODO https://github.com/dtolnay/cxx/issues/292
    let _ = types;
    Vec::from_iter(apis)
}
