#![allow(clippy::field_reassign_with_default)]

use cxx_gen::{generate_header_and_cc, Opt};
use std::str;

const BRIDGE0: &str = r#"
    #[cxx::bridge]
    mod ffi {
        unsafe extern "C++" {
            pub fn do_cpp_thing(foo: &str);
        }
    }
"#;

#[test]
fn test_extern_c_function() {
    let opt = Opt::default();
    let source = BRIDGE0.parse().unwrap();
    let generated = generate_header_and_cc(source, &opt).unwrap();
    let output = str::from_utf8(&generated.implementation).unwrap();
    // To avoid continual breakage we won't test every byte.
    // Let's look for the major features.
    assert!(output.contains("void cxxbridge1$do_cpp_thing(::rust::Str foo)"));
}

#[test]
fn test_impl_annotation() {
    let mut opt = Opt::default();
    opt.cxx_impl_annotations = Some("ANNOTATION".to_owned());
    let source = BRIDGE0.parse().unwrap();
    let generated = generate_header_and_cc(source, &opt).unwrap();
    let output = str::from_utf8(&generated.implementation).unwrap();
    assert!(output.contains("ANNOTATION void cxxbridge1$do_cpp_thing(::rust::Str foo)"));
}

const BRIDGE1: &str = r#"
    #[cxx::bridge]
    mod ffi {
        unsafe extern "C++" {
            type CppType;
        }

        extern "Rust" {
            fn rust_method_cpp_context(self: Pin<&mut CppType>);
        }
    }
"#;

// Ensure that implementing a Rust method on a C++ type only causes generation
// of the implementation.
//
// The header should be implemented in the C++ class definition and the Rust
// implementation in the usual way.
//
// This allows for developers and crates that are generating both C++ and Rust
// code to have a C++ method implemented in Rust without having to use a
// free method and passing through the C++ "this" as an argument.
#[test]
fn test_extern_rust_method_on_c_type() {
    let opt = Opt::default();
    let source = BRIDGE1.parse().unwrap();
    let generated = generate_header_and_cc(source, &opt).unwrap();
    let header = str::from_utf8(&generated.header).unwrap();
    let implementation = str::from_utf8(&generated.implementation).unwrap();

    // To avoid continual breakage we won't test every byte.
    // Let's look for the major features.

    // Check that the header doesn't have the Rust method
    assert!(!header.contains("rust_method_cpp_context"));

    // Check that there is a cxxbridge to the Rust method
    assert!(implementation
        .contains("void cxxbridge1$CppType$rust_method_cpp_context(::CppType &self) noexcept;"));

    // Check that there is a implementation on the C++ class calling the Rust method
    assert!(implementation.contains("void CppType::rust_method_cpp_context() noexcept {"));
    assert!(implementation.contains("cxxbridge1$CppType$rust_method_cpp_context(*this);"));
}
