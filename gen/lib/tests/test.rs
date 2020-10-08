use cxx_gen::Opt;
use quote::quote;

#[test]
fn test_positive() {
    let rs = quote! {
        #[cxx::bridge]
        mod ffi {
            extern "C" {
                fn in_C();
            }
            extern "Rust" {
                fn in_rs();
            }
        }
    };
    let opt = Opt::default();
    let code = cxx_gen::generate_header_and_cc(rs, &opt).unwrap();
    assert!(code.header.len() > 0);
    assert!(code.implementation.len() > 0);
}

#[test]
fn test_negative() {
    let rs = quote! {};
    let opt = Opt::default();
    assert!(cxx_gen::generate_header_and_cc(rs, &opt).is_err())
}
