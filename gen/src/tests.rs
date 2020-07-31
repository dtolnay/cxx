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
