use super::Opt;
use clap::AppSettings;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

type App = clap::App<'static, 'static>;
type Arg = clap::Arg<'static, 'static>;

const USAGE: &str = "\
    cxxbridge <input>.rs              Emit .cc file for bridge to stdout
    cxxbridge <input>.rs --header     Emit .h file for bridge to stdout
    cxxbridge --header                Emit rust/cxx.h header to stdout\
";

const TEMPLATE: &str = "\
{bin} {version}
David Tolnay <dtolnay@gmail.com>
https://github.com/dtolnay/cxx

USAGE:
    {usage}

ARGS:
{positionals}
OPTIONS:
{unified}\
";

fn app() -> App {
    let mut app = App::new("cxxbridge")
        .usage(USAGE)
        .template(TEMPLATE)
        .setting(AppSettings::NextLineHelp)
        .arg(arg_input())
        .arg(arg_cxx_impl_annotations())
        .arg(arg_header())
        .arg(arg_include())
        .help_message("Print help information.")
        .version_message("Print version information.");
    if let Some(version) = option_env!("CARGO_PKG_VERSION") {
        app = app.version(version);
    }
    app
}

const INPUT: &str = "input";
const CXX_IMPL_ANNOTATIONS: &str = "cxx-impl-annotations";
const HEADER: &str = "header";
const INCLUDE: &str = "include";

pub(super) fn from_args() -> Opt {
    let matches = app().get_matches();
    Opt {
        input: matches.value_of_os(INPUT).map(PathBuf::from),
        cxx_impl_annotations: matches.value_of(CXX_IMPL_ANNOTATIONS).map(str::to_owned),
        header: matches.is_present(HEADER),
        include: matches
            .values_of(INCLUDE)
            .map_or_else(Vec::new, |v| v.map(str::to_owned).collect()),
    }
}

fn validate_utf8(arg: &OsStr) -> Result<(), OsString> {
    if arg.to_str().is_some() {
        Ok(())
    } else {
        Err(OsString::from("invalid utf-8 sequence"))
    }
}

fn arg_input() -> Arg {
    Arg::with_name(INPUT)
        .help("Input Rust source file containing #[cxx::bridge].")
        .required_unless(HEADER)
}

fn arg_cxx_impl_annotations() -> Arg {
    const HELP: &str = "\
Optional annotation for implementations of C++ function wrappers
that may be exposed to Rust. You may for example need to provide
__declspec(dllexport) or __attribute__((visibility(\"default\")))
if Rust code from one shared object or executable depends on
these C++ functions in another.
    ";
    Arg::with_name(CXX_IMPL_ANNOTATIONS)
        .long(CXX_IMPL_ANNOTATIONS)
        .takes_value(true)
        .value_name("annotation")
        .validator_os(validate_utf8)
        .help(HELP)
}

fn arg_header() -> Arg {
    Arg::with_name(HEADER)
        .long(HEADER)
        .help("Emit header with declarations only.")
}

fn arg_include() -> Arg {
    const HELP: &str = "\
Any additional headers to #include. The cxxbridge tool does not
parse or even require the given paths to exist; they simply go
into the generated C++ code as #include lines.
    ";
    Arg::with_name(INCLUDE)
        .long(INCLUDE)
        .short("i")
        .takes_value(true)
        .multiple(true)
        .validator_os(validate_utf8)
        .help(HELP)
}
