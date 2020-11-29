fn main() {
    cxx_build::bridge("src/lib.rs")
        .file("src/repro.cpp")
        .flag_if_supported("-std=c++14")
        .compile("issue-487");
}
