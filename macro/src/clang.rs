use crate::syntax::report::Errors;
use crate::syntax::Api;
use serde::Deserialize;

type Node = clang_ast::Node<Clang>;

#[derive(Deserialize)]
enum Clang {
    Unknown,
}

pub fn load(cx: &mut Errors, apis: &mut [Api]) {
    let mut variants_from_header = Vec::new();
    for api in apis {
        if let Api::Enum(enm) = api {
            if enm.variants_from_header {
                variants_from_header.push(enm);
            }
        }
    }

    if variants_from_header.is_empty() {
        return;
    }

    let _ = cx;
    unimplemented!()
}
