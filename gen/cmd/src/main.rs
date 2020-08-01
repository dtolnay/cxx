#![allow(
    clippy::cognitive_complexity,
    clippy::inherent_to_string,
    clippy::large_enum_variant,
    clippy::new_without_default,
    clippy::toplevel_ref_arg
)]

mod app;
mod gen;
mod syntax;

use gen::include;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Debug)]
struct Opt {
    input: Option<PathBuf>,
    header: bool,
    cxx_impl_annotations: Option<String>,
    include: Vec<String>,
}

fn write(content: impl AsRef<[u8]>) {
    let _ = io::stdout().lock().write_all(content.as_ref());
}

fn main() {
    let opt = app::from_args();

    let gen = gen::Opt {
        include: opt.include,
        cxx_impl_annotations: opt.cxx_impl_annotations,
    };

    match (opt.input, opt.header) {
        (Some(input), true) => write(gen::do_generate_header(&input, gen)),
        (Some(input), false) => write(gen::do_generate_bridge(&input, gen)),
        (None, true) => write(include::HEADER),
        (None, false) => unreachable!(), // enforced by required_unless
    }
}
