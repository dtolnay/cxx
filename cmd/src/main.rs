mod gen;
mod syntax;

use gen::include;
use std::io::{self, Write};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "cxxbridge",
    author,
    about = "https://github.com/dtolnay/cxx",
    usage = "\
    cxxbridge <input>.rs              Emit .cc file for bridge to stdout
    cxxbridge <input>.rs --header     Emit .h file for bridge to stdout
    cxxbridge --header                Emit cxxbridge.h header to stdout,
    cxxbridge --runtime               Emit cxxbridge.cc file to stdout", 
    help_message = "Print help information",
    version_message = "Print version information"
)]
struct Opt {
    /// Input Rust source file containing #[cxx::bridge]
    #[structopt(parse(from_os_str), required_unless_one = &["header", "runtime"])]
    input: Option<PathBuf>,

    /// Emit header with declarations only
    #[structopt(long, conflicts_with = "runtime")]
    header: bool,

    /// Emit full cxxbridge header
    #[structopt(long, conflicts_with_all = &["header", "input"])]
    runtime: bool,
}

fn write(content: impl AsRef<[u8]>) {
    let _ = io::stdout().lock().write_all(content.as_ref());
}

fn main() {
    let opt = Opt::from_args();

    match (opt.input, opt.header, opt.runtime) {
        (Some(input), true, false) => write(gen::do_generate_header(&input)),
        (Some(input), false, false) => write(gen::do_generate_bridge(&input)),
        (None, true, false) => write(include::HEADER),
        (None, false, true) => write(include::HEADER),
        _ => unreachable!(), // enforced by structopt
    }
}
