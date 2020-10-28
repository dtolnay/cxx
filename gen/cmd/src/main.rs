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
mod output;
mod syntax;

use crate::gen::error::{report, Result};
use crate::gen::fs;
use crate::gen::include::{self, Include};
use crate::output::Output;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process;

#[derive(Debug)]
struct Opt {
    input: Option<PathBuf>,
    header: bool,
    cxx_impl_annotations: Option<String>,
    include: Vec<Include>,
    outputs: Vec<Output>,
}

fn main() {
    if let Err(err) = try_main() {
        let _ = writeln!(io::stderr(), "cxxbridge: {}", report(err));
        process::exit(1);
    }
}

enum Kind {
    GeneratedHeader,
    GeneratedImplementation,
    Header,
}

fn try_main() -> Result<()> {
    let opt = app::from_args();

    let mut outputs = Vec::new();
    let mut gen_header = false;
    let mut gen_implementation = false;
    for output in opt.outputs {
        let kind = if opt.input.is_none() {
            Kind::Header
        } else if opt.header || output.ends_with(".h") {
            gen_header = true;
            Kind::GeneratedHeader
        } else {
            gen_implementation = true;
            Kind::GeneratedImplementation
        };
        outputs.push((output, kind));
    }

    let gen = gen::Opt {
        include: opt.include,
        cxx_impl_annotations: opt.cxx_impl_annotations,
        gen_header,
        gen_implementation,
        ..Default::default()
    };

    let generated_code = if let Some(input) = opt.input {
        gen::generate_from_path(&input, &gen)
    } else {
        Default::default()
    };

    for (output, kind) in outputs {
        let content = match kind {
            Kind::GeneratedHeader => &generated_code.header,
            Kind::GeneratedImplementation => &generated_code.implementation,
            Kind::Header => include::HEADER.as_bytes(),
        };
        match output {
            Output::Stdout => drop(io::stdout().write_all(content)),
            Output::File(path) => fs::write(path, content)?,
        }
    }

    Ok(())
}
