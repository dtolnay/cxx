use crate::expand;
use crate::syntax::file::Module;
use proc_macro2::TokenStream;
use quote::quote;
use syn::File;

fn bridge(cxx_bridge: TokenStream) -> String {
    let module = syn::parse2::<Module>(cxx_bridge).unwrap();
    let tokens = expand::bridge(module).unwrap();

    // TODO: Consider returning `TokenStream` and letting clients use `assert_matches!` macros
    // if Crubit publishes
    // https://github.com/google/crubit/blob/main/common/token_stream_matchers.rs as a separate
    // crate.
    let file = syn::parse2::<File>(tokens).unwrap();
    let pretty = prettyplease::unparse(&file);

    // Print the whole result in case subsequent assertions lead to a test failure.
    eprintln!("// expanded.rs - start vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv");
    eprintln!("{pretty}");
    eprintln!("// expanded.rs - end   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^");

    pretty
}

/// This is a regression test for how `UniquePtrTarget` `impl` is generated.  The regression
/// happened in a WIP version of refactoring of how generics are handled:
///
/// * Expected:     `unsafe impl<'a> ::cxx::memory::UniquePtrTarget for Borrowed<'a>`
/// * Actual/Wrong: `unsafe impl     ::cxx::memory::UniquePtrTarget for Borrowed    `
#[test]
fn test_unique_ptr_with_lifetime_parametrized_pointee_implicit_impl() {
    // Note that it is okay that the return type infers and doesn't explicitly spell out
    // the lifetime parameter of `Borrowed`.  But this lifetime parameter needs to still
    // be spelled out in `impl<'a> ... for Borrowed<'a'>` in the expansion.
    //
    // The original regression was that an incorrect refactoring started to use
    // the inner type of `UniquePtr` (i.e. `Borrowed` - without generic lifetime args)
    // in the expansion of `impl...UniquePtrTarget`.  Instead that expansion should
    // first "resolve" the inner type using `Types::resolve`.
    let rs = bridge(quote! {
        mod ffi {
            unsafe extern "C++" {
                type Borrowed<'a>;
                fn borrowed(arg: &i32) -> UniquePtr<Borrowed>;
            }
        }
    });

    assert!(rs.contains("unsafe impl<'a> ::cxx::ExternType for Borrowed<'a>"));
    assert!(rs.contains("pub fn borrowed(arg: &i32) -> ::cxx::UniquePtr<Borrowed>"));
    assert!(rs.contains("unsafe impl<'a> ::cxx::memory::UniquePtrTarget for Borrowed<'a>"));
}

/// This is a test that verifies that the lifetime arguments in `impl<'a>` comes from
/// an explicit `impl` if one is present.
#[test]
fn test_unique_ptr_with_lifetime_parametrized_pointee_explicit_impl() {
    // Note that it is okay that the return type infers and doesn't explicitly spell out
    // the lifetime parameter of `Borrowed`.  But this lifetime parameter needs to still
    // be spelled out in `impl<'a> ... for Borrowed<'a'>` in the expansion.
    //
    // The original regression was that an incorrect refactoring started to use
    // the inner type of `UniquePtr` (i.e. `Borrowed` - without generic lifetime args)
    // in the expansion of `impl...UniquePtrTarget`.  Instead that expansion should
    // first "resolve" the inner type using `Types::resolve`.
    let rs = bridge(quote! {
        mod ffi {
            unsafe extern "C++" {
                type Borrowed<'a>;
            }
            impl<'b> UniquePtr<Borrowed<'c>> {}
        }
    });

    assert!(rs.contains("unsafe impl<'a> ::cxx::ExternType for Borrowed<'a>"));
    assert!(rs.contains("unsafe impl<'b> ::cxx::memory::UniquePtrTarget for Borrowed<'c>"));
}

/// This test verifies if `String` <=> `RustString` substitution happens for `Vec<String>`.
#[test]
fn test_vec_string_return_by_value() {
    let rs = bridge(quote! {
        mod ffi {
            extern "Rust" {
                fn foo() -> Vec<String>;
            }
        }
    });

    assert!(rs.contains("__return: *mut ::cxx::private::RustVec<::cxx::alloc::string::String>"));
    assert!(rs.contains("fn __foo() -> ::cxx::alloc::vec::Vec<::cxx::alloc::string::String>"));
}

/// This test verifies if `String` <=> `RustString` substitution happens for `Vec<String>`.
#[test]
fn test_vec_string_take_by_ref() {
    let rs = bridge(quote! {
        mod ffi {
            extern "Rust" {
                fn foo(v: &Vec<String>);
            }
        }
    });

    assert!(rs.contains("v: &::cxx::private::RustVec<::cxx::alloc::string::String>"));
    assert!(rs.contains("fn __foo(v: &::cxx::alloc::vec::Vec<::cxx::alloc::string::String>)"));
}
