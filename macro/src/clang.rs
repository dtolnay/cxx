use crate::syntax::report::Errors;
use crate::syntax::Api;
use serde::Deserialize;

type Node = clang_ast::Node<Clang>;

#[derive(Deserialize)]
enum Clang {
    Unknown,
}

pub fn load(cx: &mut Errors, apis: &mut [Api]) {
    let _ = cx;
    let _ = apis;
    unimplemented!()
}
