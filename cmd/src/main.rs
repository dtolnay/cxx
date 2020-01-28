mod gen;
mod syntax;

use gen::include;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "cxxbridge", author)]
struct Opt {
    /// Input Rust source file containing #[cxx::bridge]
    #[structopt(parse(from_os_str))]
    input: Option<PathBuf>,

    /// Emit header with declarations only
    #[structopt(long)]
    header: bool,
}

fn main() {
    let opt = Opt::from_args();

    if let Some(input) = opt.input {
        let gen = if opt.header {
            gen::do_generate_header
        } else {
            gen::do_generate_bridge
        };
        let bridge = gen(&input);
        let _ = io::stdout().lock().write_all(bridge.as_ref());
    } else if opt.header {
        let header = include::HEADER;
        let _ = io::stdout().lock().write_all(header.as_ref());
    } else {
        let _ = Opt::clap().after_help("").print_help();
        process::exit(1);
    }
}
