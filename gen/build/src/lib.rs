//! The CXX code generator for constructing and compiling C++ code.
//!
//! This is intended to be used from Cargo build scripts to execute CXX's
//! C++ code generator, set up any additional compiler flags depending on
//! the use case, and make the C++ compiler invocation.
//!
//! <br>
//!
//! # Example
//!
//! Example of a canonical Cargo build script that builds a CXX bridge:
//!
//! ```no_run
//! // build.rs
//!
//! fn main() {
//!     cxx_build::bridge("src/main.rs")
//!         .file("src/demo.cc")
//!         .flag_if_supported("-std=c++11")
//!         .compile("cxxbridge-demo");
//!
//!     println!("cargo:rerun-if-changed=src/main.rs");
//!     println!("cargo:rerun-if-changed=src/demo.cc");
//!     println!("cargo:rerun-if-changed=include/demo.h");
//! }
//! ```
//!
//! A runnable working setup with this build script is shown in the *demo*
//! directory of [https://github.com/dtolnay/cxx].
//!
//! [https://github.com/dtolnay/cxx]: https://github.com/dtolnay/cxx
//!
//! <br>
//!
//! # Alternatives
//!
//! For use in non-Cargo builds like Bazel or Buck, CXX provides an
//! alternate way of invoking the C++ code generator as a standalone command
//! line tool. The tool is packaged as the `cxxbridge-cmd` crate.
//!
//! ```bash
//! $ cargo install cxxbridge-cmd  # or build it from the repo
//!
//! $ cxxbridge src/main.rs --header > path/to/mybridge.h
//! $ cxxbridge src/main.rs > path/to/mybridge.cc
//! ```

#![allow(
    clippy::inherent_to_string,
    clippy::needless_doctest_main,
    clippy::new_without_default,
    clippy::or_fun_call,
    clippy::toplevel_ref_arg
)]

mod cargo;
mod error;
mod gen;
mod paths;
mod syntax;

use crate::error::Result;
use crate::gen::error::report;
use crate::gen::{fs, Opt};
use crate::paths::TargetDir;
use cc::Build;
use std::io::{self, Write};
use std::iter;
use std::path::{Path, PathBuf};
use std::process;

/// This returns a [`cc::Build`] on which you should continue to set up any
/// additional source files or compiler flags, and lastly call its [`compile`]
/// method to execute the C++ build.
///
/// [`compile`]: https://docs.rs/cc/1.0.49/cc/struct.Build.html#method.compile
#[must_use]
pub fn bridge(rust_source_file: impl AsRef<Path>) -> Build {
    bridges(iter::once(rust_source_file))
}

/// `cxx_build::bridge` but for when more than one file contains a
/// #\[cxx::bridge\] module.
///
/// ```no_run
/// let source_files = vec!["src/main.rs", "src/path/to/other.rs"];
/// cxx_build::bridges(source_files)
///     .file("src/demo.cc")
///     .flag_if_supported("-std=c++11")
///     .compile("cxxbridge-demo");
/// ```
#[must_use]
pub fn bridges(rust_source_files: impl IntoIterator<Item = impl AsRef<Path>>) -> Build {
    let ref mut rust_source_files = rust_source_files.into_iter();
    build(rust_source_files).unwrap_or_else(|err| {
        let _ = writeln!(io::stderr(), "\n\ncxxbridge error: {}\n\n", report(err));
        process::exit(1);
    })
}

struct Project {
    out_dir: PathBuf,
    target_dir: TargetDir,
}

impl Project {
    fn init() -> Result<Self> {
        let out_dir = paths::out_dir()?;

        let target_dir = match cargo::target_dir() {
            target_dir @ TargetDir::Path(_) => target_dir,
            // Fallback if Cargo did not work.
            TargetDir::Unknown => paths::search_parents_for_target_dir(&out_dir),
        };

        Ok(Project {
            out_dir,
            target_dir,
        })
    }
}

fn build(rust_source_files: &mut dyn Iterator<Item = impl AsRef<Path>>) -> Result<Build> {
    let ref prj = Project::init()?;
    let mut build = paths::cc_build(prj);
    build.cpp(true);
    build.cpp_link_stdlib(None); // linked via link-cplusplus crate
    write_header(prj);

    for path in rust_source_files {
        generate_bridge(prj, &mut build, path.as_ref())?;
    }

    Ok(build)
}

fn write_header(prj: &Project) {
    let ref cxx_h = paths::include_dir(prj).join("rust").join("cxx.h");
    let _ = write(cxx_h, gen::include::HEADER.as_bytes());
}

fn generate_bridge(prj: &Project, build: &mut Build, rust_source_file: &Path) -> Result<()> {
    let opt = Opt::default();
    let generated = gen::generate_from_path(rust_source_file, &opt);

    let header_path = paths::out_with_extension(prj, rust_source_file, ".h")?;
    fs::create_dir_all(header_path.parent().unwrap())?;
    write(&header_path, &generated.header)?;
    paths::symlink_header(prj, &header_path, rust_source_file);

    let implementation_path = paths::out_with_extension(prj, rust_source_file, ".cc")?;
    write(&implementation_path, &generated.implementation)?;
    build.file(&implementation_path);
    Ok(())
}

fn write(path: &Path, content: &[u8]) -> Result<()> {
    if path.exists() {
        if let Ok(existing) = fs::read(path) {
            if existing == content {
                // Avoid bumping modified time with unchanged contents.
                return Ok(());
            }
        }
        let _ = fs::remove_file(path);
    } else {
        let _ = fs::create_dir_all(path.parent().unwrap());
    }
    fs::write(path, content)?;
    Ok(())
}
