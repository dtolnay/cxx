const EXPECTED: &str = "\
cxxbridge $VERSION
David Tolnay <dtolnay@gmail.com>
https://github.com/dtolnay/cxx

USAGE:
    cxxbridge <input>.rs              Emit .cc file for bridge to stdout
    cxxbridge <input>.rs --header     Emit .h file for bridge to stdout
    cxxbridge --header                Emit \"rust/cxx.h\" header to stdout

ARGS:
    <input>
            Input Rust source file containing #[cxx::bridge].

OPTIONS:
        --cfg <name=\"value\" | name[=true] | name=false>
            Compilation configuration matching what will be used to build
            the Rust side of the bridge.

        --cxx-impl-annotations <annotation>
            Optional annotation for implementations of C++ function wrappers
            that may be exposed to Rust. You may for example need to provide
            __declspec(dllexport) or __attribute__((visibility(\"default\")))
            if Rust code from one shared object or executable depends on
            these C++ functions in another.
            
            If you need to use this option, you might also consider similarly
            defining either or both of the preprocessor symbols CXX_RS_EXPORT
            and CXX_CPP_EXPORT when compiling cxx.cc. CXX_RS_EXPORT
            attaches to symbols exported from cxx.cc which are typically used
            by Rust code; CXX_CPP_EXPORT attaches to symbols which are used
            by C++ code.

    -h, --help
            Print help information.

        --header
            Emit header with declarations only. Optional if using `-o` with
            a path ending in `.h`.

    -i, --include <include>
            Any additional headers to #include. The cxxbridge tool does not
            parse or even require the given paths to exist; they simply go
            into the generated C++ code as #include lines.

    -o, --output <output>
            Path of file to write as output. Output goes to stdout if -o is
            not specified.

    -V, --version
            Print version information.
";

#[test]
fn test_help() {
    let mut app = super::app();
    let mut out = Vec::new();
    app.write_long_help(&mut out).unwrap();
    let help = String::from_utf8(out).unwrap();
    let version = option_env!("CARGO_PKG_VERSION").unwrap_or_default();
    let expected = EXPECTED.replace("$VERSION", version);
    assert_eq!(help, expected);
}

#[test]
fn test_cli() {
    let app = super::app();
    app.debug_assert();
}
