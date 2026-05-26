mod cpp_compile;

use indoc::indoc;
use proc_macro2::TokenStream;
use quote::quote;

use cpp_compile::CompilationResult;

#[must_use]
fn compile_test(rust_code: TokenStream, include_h: &str) -> CompilationResult {
    let test = cpp_compile::Test::new(rust_code);
    test.write_file("include.h", include_h);
    test.compile()
}

/// This is a regression test for `static_assert(::rust::is_complete...)`
/// which we started to emit in <https://github.com/dtolnay/cxx/commit/534627667>
#[test]
fn test_unique_ptr_of_incomplete_foward_declared_pointee() {
    let compiled = compile_test(
        quote! {
            #[cxx::bridge]
            mod ffi {
                unsafe extern "C++" {
                    include!("include.h");
                    type ForwardDeclaredType;
                }
                impl UniquePtr<ForwardDeclaredType> {}
            }
        },
        indoc! {"
            class ForwardDeclaredType;
        "},
    );
    assert!(compiled
        .expect_single_error()
        .contains("definition of `::ForwardDeclaredType` is required"))
}

#[test]
fn test_safe_shared_extern_with_wrong_field_type() {
    // Type mismatch in one of the struct fields
    let compiled = compile_test(
        quote! {
            #[cxx::bridge]
            mod ffi {
                #[safe_shared_extern]
                struct A {
                    a: u8,
                }

                extern "C++" {
                    include!("include.h");
                }
            }
        },
        indoc! {"
            struct A {
                signed char a;
            };
        "},
    );
    assert!(compiled.expect_single_error().contains("wrong field type"));
}

#[test]
fn test_safe_shared_extern_with_missing_field_cpp() {
    // Missing field in C++
    let compiled = compile_test(
        quote! {
            #[cxx::bridge]
            mod ffi {
                #[safe_shared_extern]
                struct A {
                    a: u8,
                    b: u8,
                }

                extern "C++" {
                    include!("include.h");
                }
            }
        },
        indoc! {"
            struct A {
                unsigned char b;
            };
        "},
    );
    // The error messages vary depending on the compiler
    assert!(!compiled.error_lines().is_empty());
}

#[test]
fn test_safe_shared_extern_with_missing_field_rust() {
    // Missing field in Rust, resulting in different structure size
    let compiled = compile_test(
        quote! {
            #[cxx::bridge]
            mod ffi {
                #[safe_shared_extern]
                struct A {
                    a: i32,
                }

                extern "C++" {
                    include!("include.h");
                }
            }
        },
        indoc! {"
            #include <cstdint>

            struct A {
                int32_t a;
                int32_t b;
            };
        "},
    );
    assert!(compiled.expect_single_error().contains(
        "unexpected struct size; note that structs with padding in the layout are not supported"
    ));

    // Missing field in Rust, which fits in the padding, so the structure size is the same.
    // Since we cannot reliably detect this, we just refuse any struct with padding.
    let compiled = compile_test(
        quote! {
            #[cxx::bridge]
            mod ffi {
                #[safe_shared_extern]
                struct A {
                    a: u8,
                    c: i16,
                }

                extern "C++" {
                    include!("include.h");
                }
            }
        },
        indoc! {"
            #include <cstdint>

            struct A {
                unsigned char a;
                unsigned char b;
                int16_t c;
            };
        "},
    );
    assert!(compiled.expect_single_error().contains(
        "unexpected struct size; note that structs with padding in the layout are not supported"
    ));
}

#[test]
fn test_safe_shared_extern_with_wrong_field_order() {
    // Fields appear in a wrong order
    let compiled = compile_test(
        quote! {
            #[cxx::bridge]
            mod ffi {
                #[safe_shared_extern]
                struct A {
                    b: u8,
                    a: u8,
                }

                extern "C++" {
                    include!("include.h");
                }
            }
        },
        indoc! {"
            struct A {
                unsigned char a;
                unsigned char b;
            };
        "},
    );
    assert!(compiled
        .expect_single_error()
        .contains("wrong fields order"));
}

#[test]
fn test_safe_shared_extern_with_wrong_field_visibility() {
    // A field is declared private on the C++ side
    let compiled = compile_test(
        quote! {
            #[cxx::bridge]
            mod ffi {
                #[safe_shared_extern]
                struct A {
                    a: u8,
                }

                extern "C++" {
                    include!("include.h");
                }
            }
        },
        indoc! {"
            struct A {
            private:
                unsigned char a;
            };
        "},
    );
    // The exact message varies from compiler to compiler
    let err_msg = compiled.error_lines().join("\n");
    assert!(
        err_msg.contains("is private")
            || err_msg.contains("is a private member")
            || err_msg.contains("cannot access private member")
    );
}

#[test]
fn test_safe_shared_extern_with_errorneously_nontrivial_struct() {
    let compiled = compile_test(
        quote! {
            #[cxx::bridge]
            mod ffi {
                #[safe_shared_extern]
                struct A {
                    a: u8,
                }

                extern "C++" {
                    include!("include.h");

                    // `A` must be POD because it is used as a return value.
                    unsafe fn a() -> A;
                }
            }
        },
        indoc! {"
            struct A {
                unsigned char a;
                ~A() {}
            };

            A a();
        "},
    );
    assert!(compiled
        .expect_single_error()
        .contains("cxx does not yet support non-trivial structs as safe shared extern"));
}

/// Despite the fact that the C++ struct is non-trivial (has a destructor),
/// since it is never used as a return value or a by-value parameter, we
/// can support it as a `#[safe_shared_extern]`.
///
/// However, this is not implemented yet: we currently unconditionally reject
/// any non-trivial structs.
#[test]
#[ignore = "non-trivial shared structs are currently rejected unconditionally; not implemented yet"]
fn test_safe_shared_extern_with_nontrivial_struct() {
    let compiled = compile_test(
        quote! {
            #[cxx::bridge]
            mod ffi {
                #[safe_shared_extern]
                struct A {
                    a: u8,
                }

                extern "C++" {
                    include!("include.h");
                }
            }
        },
        indoc! {"
            struct A {
                unsigned char a;
                ~A() {}
            };

            A a();
        "},
    );
    compiled.assert_success();
}

#[test]
fn test_safe_shared_extern_with_typedefed() {
    let compiled = compile_test(
        quote! {
            #[cxx::bridge]
            mod ffi {
                #[safe_shared_extern]
                struct A {
                    a: u8,
                }

                extern "C++" {
                    include!("include.h");

                    unsafe fn a() -> A;
                }
            }
        },
        indoc! {"
            typedef struct {
                unsigned char a;
            } A;

            A a();
        "},
    );
    compiled.assert_success();
}
