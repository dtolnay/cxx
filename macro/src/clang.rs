use crate::syntax::report::Errors;
use crate::syntax::Api;
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::PathBuf;

const CXX_CLANG_AST: &str = "CXX_CLANG_AST";

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

    let span = match variants_from_header.get(0) {
        None => return,
        Some(enm) => enm.variants_from_header_attr.clone().unwrap(),
    };

    let ast_dump_path = match env::var_os(CXX_CLANG_AST) {
        Some(ast_dump_path) => PathBuf::from(ast_dump_path),
        None => {
            let msg = format!(
                "environment variable ${} has not been provided",
                CXX_CLANG_AST,
            );
            return cx.error(span, msg);
        }
    };

    let ast_dump_bytes = match fs::read(&ast_dump_path) {
        Ok(ast_dump_bytes) => ast_dump_bytes,
        Err(error) => {
            let msg = format!("failed to read {}: {}", ast_dump_path.display(), error);
            return cx.error(span, msg);
        }
    };

    let root: Node = match serde_json::from_slice(&ast_dump_bytes) {
        Ok(root) => root,
        Err(error) => {
            let msg = format!("failed to read {}: {}", ast_dump_path.display(), error);
            return cx.error(span, msg);
        }
    };

    unimplemented!()
}
