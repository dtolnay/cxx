mod gen;
mod syntax;

use std::io::{self, Write};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "cxxbridge", author)]
struct Opt {
    /// Input Rust source file containing #[cxx::bridge]
    #[structopt(parse(from_os_str))]
    input: PathBuf,

    /// Emit header with declarations only
    #[structopt(long)]
    header: bool,
}

fn main() {
    let opt = Opt::from_args();
    let gen = if opt.header {
        gen::do_generate_header
    } else {
        gen::do_generate_bridge
    };
    let bridge = gen(&opt.input);
    let _ = io::stdout().lock().write_all(bridge.as_ref());
}
