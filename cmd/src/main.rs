mod gen;
mod syntax;

use gen::include::get_full_cxxbridge;
use std::io::{self, Error, ErrorKind, Write};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "cxxbridge", author)]
struct Opt {
    /// Input Rust source file containing #[cxx::bridge]
    #[structopt(parse(from_os_str))]
    input: Option<PathBuf>,

    /// Emit header with declarations only. If no input is specified, emit cxxbridge.h
    #[structopt(long)]
    header: bool,
}

fn main() -> Result<(), Error> {
    let opt = Opt::from_args();

    if let Some(input) = opt.input {
        let gen = if opt.header {
            gen::do_generate_header
        } else {
            gen::do_generate_bridge
        };
        let bridge = gen(&input);
        io::stdout().lock().write_all(bridge.as_ref())
    } else if opt.header {
        io::stdout().lock().write_all(get_full_cxxbridge().as_ref())
    } else {
        let mut clap = Opt::clap().after_help("");
        clap.print_help().or(Err(Error::new(ErrorKind::Other, "Failed to write help")))
    }
}
