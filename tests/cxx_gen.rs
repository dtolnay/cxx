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

const BRIDGE1: &str = r"
    #[cxx::bridge]
    mod ffi {
        #[derive(JsgStruct)]
        struct MyStruct {
            field1: i32,
            field2: String,
        }
    }
";

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
    let opt = Opt {
        cxx_impl_annotations: Some("ANNOTATION".to_owned()),
        ..Default::default()
    };
    let source = BRIDGE0.parse().unwrap();
    let generated = generate_header_and_cc(source, &opt).unwrap();
    let output = str::from_utf8(&generated.implementation).unwrap();
    assert!(output.contains("ANNOTATION void cxxbridge1$do_cpp_thing(::rust::Str foo)"));
}

#[test]
fn test_jsg_struct_derive() {
    let opt = Opt::default();
    let source = BRIDGE1.parse().unwrap();
    let generated = generate_header_and_cc(source, &opt).unwrap();
    let output = str::from_utf8(&generated.header).unwrap();
    assert!(output.contains("JSG_STRUCT(field1, field2);"));
    assert!(output.contains("jsg.h"));
}
