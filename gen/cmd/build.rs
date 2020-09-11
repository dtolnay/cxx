use std::io::{self, Write};
use std::path::Path;
use std::process;

const NOSYMLINK: &str = "
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
When building `cxx` from a git clone, git's symlink support needs
to be enabled on platforms that have it off by default (Windows).
Either use:

   $ git config --global core.symlinks true

prior to cloning, or else use:

   $ git clone -c core.symlinks=true ...

for the clone.

Symlinks are only required for local development, not for building
`cxx` as a (possibly transitive) dependency from crates.io.
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
";

fn main() {
    if !Path::new("src/syntax/mod.rs").exists() {
        let _ = io::stderr().lock().write_all(NOSYMLINK.as_bytes());
        process::exit(1);
    }
}
