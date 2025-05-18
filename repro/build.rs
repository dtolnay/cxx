use cxx_build::CFG;

fn main() {
    CFG.include_prefix = "repro";

    cxx_build::bridge("src/lib.rs")
        .file("src/helper.cc")
        .compile("waldemarkunkel");
}
