#[allow(unused_attributes)]
#[rustversion::attr(not(nightly), ignore)]
#[cfg_attr(skip_ui_tests, ignore)]
#[cfg_attr(miri, ignore)]
#[test]
fn ui() {
    let check = trybuild::TestCases::new();
    check.compile_fail("tests/ui/*.rs");
    // Tests that require cargo build instead of cargo check
    let build = trybuild::TestCases::new();
    // Having passing cases forces cargo build instead of cargo check
    build.pass("tests/ui_pass/*.rs");
    build.compile_fail("tests/ui_build/*.rs");
}
