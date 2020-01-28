mod gen;
mod syntax;

use gen::include;
use std::io::{self, Write};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "cxxbridge", author)]
struct Opt {
    /// Input Rust source file containing #[cxx::bridge]
    #[structopt(parse(from_os_str), required_unless = "header")]
    input: Option<PathBuf>,

    /// Emit header with declarations only
    #[structopt(long)]
    header: bool,
}

fn write(content: impl AsRef<[u8]>) {
    let _ = io::stdout().lock().write_all(content.as_ref());
}

fn main() {
    let opt = Opt::from_args();

    match (opt.input, opt.header) {
        (Some(input), true) => write(gen::do_generate_header(&input)),
        (Some(input), false) => write(gen::do_generate_bridge(&input)),
        (None, true) => write(include::HEADER),
        (None, false) => unreachable!(), // enforced by required_unless
    }
}
