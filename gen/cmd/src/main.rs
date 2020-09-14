#![allow(
    clippy::cognitive_complexity,
    clippy::inherent_to_string,
    clippy::large_enum_variant,
    clippy::new_without_default,
    clippy::or_fun_call,
    clippy::toplevel_ref_arg
)]

mod app;
mod gen;
mod syntax;

use gen::error::{report, Result};
use gen::{fs, include};
use std::io::{self, Write};
use std::path::PathBuf;
use std::process;

#[derive(Debug)]
struct Opt {
    input: Option<PathBuf>,
    header: bool,
    cxx_impl_annotations: Option<String>,
    include: Vec<String>,
    output: Output,
}

#[derive(Debug)]
enum Output {
    Stdout,
    File(PathBuf),
}

fn main() {
    if let Err(err) = try_main() {
        let _ = writeln!(io::stderr(), "cxxbridge: {}", report(err));
        process::exit(1);
    }
}

fn try_main() -> Result<()> {
    let opt = app::from_args();

    let gen = gen::Opt {
        include: opt.include,
        cxx_impl_annotations: opt.cxx_impl_annotations,
        gen_header: opt.header,
        gen_implementation: !opt.header,
    };

    let content;
    let content = match (opt.input, opt.header) {
        (Some(input), true) => {
            content = gen::generate_from_path(&input, &gen).header;
            content.as_slice()
        }
        (Some(input), false) => {
            content = gen::generate_from_path(&input, &gen).implementation;
            content.as_slice()
        }
        (None, true) => include::HEADER.as_bytes(),
        (None, false) => unreachable!(), // enforced by required_unless
    };

    match opt.output {
        Output::Stdout => drop(io::stdout().write_all(content)),
        Output::File(path) => fs::write(path, content)?,
    }

    Ok(())
}
