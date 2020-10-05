use cxx_gen::{generate_header_and_cc, Opt};

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
    let opt = Opt::default();
    let source = CPP_EXAMPLE.parse().unwrap();
    let output = generate_header_and_cc(source, &opt).unwrap();
    let output = std::str::from_utf8(&output.implementation).unwrap();
    // To avoid continual breakage we won't test every byte.
    // Let's look for the major features.
    assert!(output.contains("void cxxbridge04$do_cpp_thing(::rust::Str::Repr foo)"));
}

#[test]
fn test_annotation() {
    let mut opt = Opt::default();
    opt.cxx_impl_annotations = Some("ANNOTATION".to_owned());
    let source = CPP_EXAMPLE.parse().unwrap();
    let output = generate_header_and_cc(source, &opt).unwrap();
    let output = std::str::from_utf8(&output.implementation).unwrap();
    assert!(output.contains("ANNOTATION void cxxbridge04$do_cpp_thing(::rust::Str::Repr foo)"));
}
