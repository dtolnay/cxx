#![allow(
    clippy::cognitive_complexity,
    clippy::inherent_to_string,
    clippy::large_enum_variant,
    clippy::new_without_default,
    clippy::toplevel_ref_arg
)]

mod gen;
mod syntax;

use gen::include;
use std::io::{self, Write};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "cxxbridge",
    author = "David Tolnay <dtolnay@gmail.com>",
    about = "https://github.com/dtolnay/cxx",
    usage = "\
    cxxbridge <input>.rs              Emit .cc file for bridge to stdout
    cxxbridge <input>.rs --header     Emit .h file for bridge to stdout
    cxxbridge --header                Emit rust/cxx.h header to stdout",
    help_message = "Print help information",
    version_message = "Print version information"
)]
struct Opt {
    /// Input Rust source file containing #[cxx::bridge]
    #[structopt(parse(from_os_str), required_unless = "header")]
    input: Option<PathBuf>,

    /// Emit header with declarations only
    #[structopt(long)]
    header: bool,

    /// Any additional headers to #include
    #[structopt(short, long)]
    include: Vec<String>,
}

fn write(content: impl AsRef<[u8]>) {
    let _ = io::stdout().lock().write_all(content.as_ref());
}

fn main() {
    let opt = Opt::from_args();

    let gen = gen::Opt {
        include: opt.include,
    };

    match (opt.input, opt.header) {
        (Some(input), true) => write(gen::do_generate_header(&input, gen)),
        (Some(input), false) => write(gen::do_generate_bridge(&input, gen)),
        (None, true) => write(include::HEADER),
        (None, false) => unreachable!(), // enforced by required_unless
    }
}
