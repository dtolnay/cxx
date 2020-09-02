use crate::gen::{generate_from_string, Opt};

const CPP_EXAMPLE: &str = r#"
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
        gen_header: false,
        gen_implementation: true,
    };
    let output = generate_from_string(CPP_EXAMPLE, &opts).unwrap();
    let output = std::str::from_utf8(&output.implementation).unwrap();
    // To avoid continual breakage we won't test every byte.
    // Let's look for the major features.
    assert!(output.contains("void cxxbridge04$do_cpp_thing(::rust::Str::Repr foo)"));
}

#[test]
fn test_annotation() {
    let opts = Opt {
        include: Vec::new(),
        cxx_impl_annotations: Some("ANNOTATION".to_string()),
        gen_header: false,
        gen_implementation: true,
    };
    let output = generate_from_string(CPP_EXAMPLE, &opts).unwrap();
    let output = std::str::from_utf8(&output.implementation).unwrap();
    assert!(output.contains("ANNOTATION void cxxbridge04$do_cpp_thing(::rust::Str::Repr foo)"));
}
