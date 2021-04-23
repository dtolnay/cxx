use crate::syntax::attrs::OtherAttrs;
use crate::syntax::namespace::Namespace;
use crate::syntax::report::Errors;
use crate::syntax::{Api, Discriminant, Doc, Enum, ForeignName, Pair, Variant};
use proc_macro2::{Delimiter, Group, Ident, TokenStream};
use quote::{format_ident, quote};
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

const CXX_CLANG_AST: &str = "CXX_CLANG_AST";

type Node = clang_ast::Node<Clang>;

#[derive(Deserialize)]
enum Clang {
    NamespaceDecl(NamespaceDecl),
    EnumDecl(EnumDecl),
    EnumConstantDecl(EnumConstantDecl),
    ImplicitCastExpr,
    ConstantExpr(ConstantExpr),
    Unknown,
}

#[derive(Deserialize)]
struct NamespaceDecl {
    name: Option<String>,
}

#[derive(Deserialize)]
struct EnumDecl {
    name: Option<String>,
}

#[derive(Deserialize)]
struct EnumConstantDecl {
    name: String,
}

#[derive(Deserialize)]
struct ConstantExpr {
    value: String,
}

pub fn load(cx: &mut Errors, apis: &mut [Api]) {
    let ref mut variants_from_header = Vec::new();
    for api in apis {
        if let Api::Enum(enm) = api {
            if enm.variants_from_header {
                if enm.variants.is_empty() {
                    variants_from_header.push(enm);
                } else {
                    let span = span_for_enum_error(enm);
                    cx.error(
                        span,
                        "enum with #![variants_from_header] must be written with no explicit variants",
                    );
                }
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

    let ref root: Node = match serde_json::from_slice(&ast_dump_bytes) {
        Ok(root) => root,
        Err(error) => {
            let msg = format!("failed to read {}: {}", ast_dump_path.display(), error);
            return cx.error(span, msg);
        }
    };

    let ref mut namespace = Vec::new();
    traverse(cx, root, namespace, variants_from_header, None);

    for enm in variants_from_header {
        if enm.variants.is_empty() {
            let span = &enm.variants_from_header_attr;
            let mut msg = "failed to find any C++ definition of enum ".to_owned();
            for name in &enm.name.namespace {
                msg += &name.to_string();
                msg += "::";
            }
            msg += &enm.name.cxx.to_string();
            cx.error(span, msg);
        }
    }
}

fn traverse<'a>(
    cx: &mut Errors,
    node: &'a Node,
    namespace: &mut Vec<&'a str>,
    variants_from_header: &mut [&mut Enum],
    mut idx: Option<usize>,
) {
    match &node.kind {
        Clang::NamespaceDecl(decl) => {
            let name = match &decl.name {
                Some(name) => name,
                // Can ignore enums inside an anonymous namespace.
                None => return,
            };
            namespace.push(name);
            idx = None;
        }
        Clang::EnumDecl(decl) => {
            let name = match &decl.name {
                Some(name) => name,
                None => return,
            };
            idx = None;
            for (i, enm) in variants_from_header.iter().enumerate() {
                if enm.name.cxx == **name && enm.name.namespace.iter().eq(&*namespace) {
                    idx = Some(i);
                    break;
                }
            }
            if idx.is_none() {
                return;
            }
        }
        Clang::EnumConstantDecl(decl) => {
            if let Some(idx) = idx {
                let enm = &mut *variants_from_header[idx];
                let span = enm
                    .variants_from_header_attr
                    .as_ref()
                    .unwrap()
                    .path
                    .get_ident()
                    .unwrap()
                    .span();
                let cxx_name = match ForeignName::parse(&decl.name, span) {
                    Ok(foreign_name) => foreign_name,
                    Err(_) => {
                        let span = &enm.variants_from_header_attr;
                        let msg = format!("unsupported C++ variant name: {}", decl.name);
                        return cx.error(span, msg);
                    }
                };
                let rust_name: Ident = match syn::parse_str(&decl.name) {
                    Ok(ident) => ident,
                    Err(_) => format_ident!("__Variant{}", enm.variants.len()),
                };
                let discriminant = match discriminant_value(&node.inner) {
                    ParsedDiscriminant::Constant(discriminant) => discriminant,
                    ParsedDiscriminant::Successor => match enm.variants.last() {
                        None => Discriminant::zero(),
                        Some(last) => match last.discriminant.checked_succ() {
                            Some(discriminant) => discriminant,
                            None => {
                                let span = &enm.variants_from_header_attr;
                                let msg = format!(
                                    "overflow processing discriminant value for variant: {}",
                                    decl.name,
                                );
                                return cx.error(span, msg);
                            }
                        },
                    },
                    ParsedDiscriminant::Fail => {
                        let span = &enm.variants_from_header_attr;
                        let msg = format!(
                            "failed to obtain discriminant value for variant: {}",
                            decl.name,
                        );
                        cx.error(span, msg);
                        Discriminant::zero()
                    }
                };
                enm.variants.push(Variant {
                    doc: Doc::new(),
                    attrs: OtherAttrs::none(),
                    name: Pair {
                        namespace: Namespace::ROOT,
                        cxx: cxx_name,
                        rust: rust_name,
                    },
                    discriminant,
                    expr: None,
                });
            }
        }
        _ => {}
    }
    for inner in &node.inner {
        traverse(cx, inner, namespace, variants_from_header, idx);
    }
    if let Clang::NamespaceDecl(_) = &node.kind {
        let _ = namespace.pop().unwrap();
    }
}

enum ParsedDiscriminant {
    Constant(Discriminant),
    Successor,
    Fail,
}

fn discriminant_value(mut clang: &[Node]) -> ParsedDiscriminant {
    if clang.is_empty() {
        // No discriminant expression provided; use successor of previous
        // descriminant.
        return ParsedDiscriminant::Successor;
    }

    loop {
        if clang.len() != 1 {
            return ParsedDiscriminant::Fail;
        }

        let node = &clang[0];
        match &node.kind {
            Clang::ImplicitCastExpr => clang = &node.inner,
            Clang::ConstantExpr(expr) => match Discriminant::from_str(&expr.value) {
                Ok(discriminant) => return ParsedDiscriminant::Constant(discriminant),
                Err(_) => return ParsedDiscriminant::Fail,
            },
            _ => return ParsedDiscriminant::Fail,
        }
    }
}

fn span_for_enum_error(enm: &Enum) -> TokenStream {
    let enum_token = enm.enum_token;
    let mut brace_token = Group::new(Delimiter::Brace, TokenStream::new());
    brace_token.set_span(enm.brace_token.span);
    quote!(#enum_token #brace_token)
}