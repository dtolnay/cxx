const EXPECTED: &str = "\
cxxbridge 0.3.7
David Tolnay <dtolnay@gmail.com>
https://github.com/dtolnay/cxx

USAGE:
    cxxbridge <input>.rs              Emit .cc file for bridge to stdout
    cxxbridge <input>.rs --header     Emit .h file for bridge to stdout
    cxxbridge --header                Emit rust/cxx.h header to stdout

ARGS:
    <input>
            Input Rust source file containing #[cxx::bridge].

OPTIONS:
        --cxx-impl-annotations <annotation>
            Optional annotation for implementations of C++ function wrappers
            that may be exposed to Rust. You may for example need to provide
            __declspec(dllexport) or __attribute__((visibility(\"default\")))
            if Rust code from one shared object or executable depends on
            these C++ functions in another.

        --header
            Emit header with declarations only.

    -h, --help
            Print help information.

    -i, --include <include>...
            Any additional headers to #include. The cxxbridge tool does not
            parse or even require the given paths to exist; they simply go
            into the generated C++ code as #include lines.

    -V, --version
            Print version information.

";

#[test]
fn test_help() {
    let mut app = super::app();
    let mut out = Vec::new();
    app.write_long_help(&mut out).unwrap();
    let help = String::from_utf8(out).unwrap();
    assert_eq!(help, EXPECTED);
}
