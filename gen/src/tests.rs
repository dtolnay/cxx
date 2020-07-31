use crate::gen::{generate, Opt};

const CPP_EXAMPLE: &'static str = r#"
    #[cxx::bridge]
    mod ffi {
        extern "C" {
            pub fn do_cpp_thing(foo: &str);
        }
    }
    "#;

fn generate_for_test(input: &str, header: bool, opts: Opt) -> String {
    let output = generate(input, opts, header).unwrap();
    std::str::from_utf8(&output).unwrap().to_string()
}

#[test]
fn test_cpp() {
    let opts = Opt {
        include: Vec::new(),
        cxx_impl_annotations: None,
        skip_definitions: false,
    };
    let output = generate_for_test(CPP_EXAMPLE, false, opts);
    // To avoid continual breakage we won't test every byte.
    // Let's look for the major features.
    assert!(output.contains("void cxxbridge03$do_cpp_thing(::rust::Str::Repr foo)"));
}

#[test]
fn test_annotation() {
    let opts = Opt {
        include: Vec::new(),
        cxx_impl_annotations: Some("ANNOTATION".to_string()),
        skip_definitions: false,
    };
    let output = generate_for_test(CPP_EXAMPLE, false, opts);
    assert!(output.contains("ANNOTATION void cxxbridge03$do_cpp_thing(::rust::Str::Repr foo)"));
}

const STRUCT_EXAMPLE: &'static str = r#"
    #[cxx::bridge]
    mod ffi {
        struct bob {
            bob_field: i32,
        }
        enum norman {
            norman_variant
        }
        extern "C" {
            pub fn do_cpp_thing(foo: &bob, bar: norman);
        }
    }
    "#;
#[test]
fn test_no_definitions() {
    let opts = Opt {
        include: Vec::new(),
        cxx_impl_annotations: None,
        skip_definitions: false,
    };
    let output = generate_for_test(STRUCT_EXAMPLE, false, opts.clone());
    assert!(output.contains("bob_field"));
    assert!(output.contains("norman_variant"));
    let output = generate_for_test(STRUCT_EXAMPLE, true, opts);
    assert!(output.contains("bob_field"));
    assert!(output.contains("norman_variant"));
}

#[test]
fn test_skip_definitions() {
    let opts = Opt {
        include: Vec::new(),
        cxx_impl_annotations: None,
        skip_definitions: true,
    };
    let output = generate_for_test(STRUCT_EXAMPLE, false, opts.clone());
    assert!(!output.contains("bob_field"));
    assert!(!output.contains("norman_variant"));
    let output = generate_for_test(STRUCT_EXAMPLE, true, opts);
    assert!(output.contains("bob_field"));
    assert!(output.contains("norman_variant"));
}
