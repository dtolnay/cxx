// Functionality that is shared between the cxx_build::bridge entry point and
// the cxxbridge CLI command.

mod error;
mod find;
pub(super) mod include;
pub(super) mod out;
mod write;

use self::error::{format_err, Error, Result};
use crate::syntax::namespace::Namespace;
use crate::syntax::report::Errors;
use crate::syntax::{self, check, Types};
use std::fs;
use std::path::Path;
use syn::Item;

struct Input {
    namespace: Namespace,
    module: Vec<Item>,
}

#[derive(Default)]
pub(super) struct Opt {
    /// Any additional headers to #include
    pub include: Vec<String>,
    /// Whether to set __attribute__((visibility("default")))
    /// or similar annotations on function implementations.
    pub cxx_impl_annotations: Option<String>,
}

pub(super) fn do_generate_bridge(path: &Path, opt: Opt) -> Vec<u8> {
    let header = false;
    generate_from_path(path, opt, header)
}

pub(super) fn do_generate_header(path: &Path, opt: Opt) -> Vec<u8> {
    let header = true;
    generate_from_path(path, opt, header)
}

fn generate_from_path(path: &Path, opt: Opt, header: bool) -> Vec<u8> {
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(err) => format_err(path, "", Error::Io(err)),
    };
    match generate(&source, opt, header) {
        Ok(out) => out,
        Err(err) => format_err(path, &source, err),
    }
}

fn generate(source: &str, opt: Opt, header: bool) -> Result<Vec<u8>> {
    proc_macro2::fallback::force();
    let ref mut errors = Errors::new();
    let syntax = syn::parse_file(&source)?;
    let bridge = find::find_bridge_mod(syntax)?;
    let ref namespace = bridge.namespace;
    let ref apis = syntax::parse_items(errors, bridge.module);
    let ref types = Types::collect(errors, apis);
    errors.propagate()?;
    check::typecheck(errors, namespace, apis, types);
    errors.propagate()?;
    let out = write::gen(namespace, apis, types, opt, header);
    Ok(out.content())
}

#[cfg(test)]
mod tests {
    use crate::gen::{generate, Opt};

    const CPP_EXAMPLE: &'static str = r#"
        #[cxx::bridge]
        mod ffi {
            extern "C" {
                pub fn do_cpp_thing(foo: &str);
            }
        }
        "#;

    #[test]
    fn test_cpp() {
        let opts = Opt {
            include: Vec::new(),
            cxx_impl_annotations: None,
        };
        let output = generate(CPP_EXAMPLE, opts, false).unwrap();
        let output = std::str::from_utf8(&output).unwrap();
        // To avoid continual breakage we won't test every byte.
        // Let's look for the major features.
        assert!(output.contains("void cxxbridge03$do_cpp_thing(::rust::Str::Repr foo)"));
    }

    #[test]
    fn test_annotation() {
        let opts = Opt {
            include: Vec::new(),
            cxx_impl_annotations: Some("ANNOTATION".to_string()),
        };
        let output = generate(CPP_EXAMPLE, opts, false).unwrap();
        let output = std::str::from_utf8(&output).unwrap();
        assert!(output.contains("ANNOTATION void cxxbridge03$do_cpp_thing(::rust::Str::Repr foo)"));
    }
}
