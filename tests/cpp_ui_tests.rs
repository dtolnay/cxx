mod cpp_compile;

use indoc::indoc;
use quote::quote;

/// This is a regression test for `static_assert(::rust::is_complete...)`
/// which we started to emit in <https://github.com/dtolnay/cxx/commit/534627667>
#[test]
fn test_unique_ptr_of_incomplete_foward_declared_pointee() {
    let test = cpp_compile::Test::new(quote! {
        #[cxx::bridge]
        mod ffi {
            unsafe extern "C++" {
                include!("include.h");
                type ForwardDeclaredType;
            }
            impl UniquePtr<ForwardDeclaredType> {}
        }
    });
    test.write_file(
        "include.h",
        indoc! {"
            class ForwardDeclaredType;
        "},
    );
    let err_msg = test.compile().expect_single_error();
    assert!(err_msg.contains("definition of `::ForwardDeclaredType` is required"));
}

#[test]
fn test_str_rejects_non_utf8() {
    let test = cpp_compile::Test::new(quote! {
        #[cxx::bridge]
        mod ffi {
            extern "Rust" {
                fn dummy(s: &str);
            }
        }
    });
    test.write_file(
        "cxx_bridge.generated.cc",
        indoc! {r#"
            #include "cxx_bridge.generated.h"

            #ifdef __cpp_char8_t
            using rust::operator""_utf8;
            inline void must_fail() {
                rust::Str s{u8"test\xff"_utf8};
            }
            #endif
        "#},
    );
    let err_msg = test.compile().expect_single_error();
    println!("error message: {err_msg}");
    assert!(err_msg.contains("consteval function"), "unexpected error: {err_msg}");
}

#[test]
fn test_str_rejects_non_constexpr_variable() {
    let test = cpp_compile::Test::new(quote! {
        #[cxx::bridge]
        mod ffi {
            extern "Rust" {
                fn dummy(s: &str);
            }
        }
    });
    test.write_file(
        "cxx_bridge.generated.cc",
        indoc! {r#"
            #include "cxx_bridge.generated.h"

            #ifdef __cpp_char8_t
            inline void must_fail() {
                rust::Str s{u8"test\xff"};
            }
            #endif
        "#},
    );
    let err_msg = test.compile().expect_single_error();
    println!("error message: {err_msg}");
    assert!(err_msg.contains("no matching constructor"), "unexpected error: {err_msg}");
}
