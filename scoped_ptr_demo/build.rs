fn main() {
    cxx_build::bridge("src/main.rs")
        .file("src/demo.cc")
        .flag_if_supported("-std=c++17")
        .compile("scoped-ptr-demo");
}
