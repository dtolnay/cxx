use std::collections::BTreeMap;
use std::env;
use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Default)]
pub struct Crate {
    pub include_prefix: Option<PathBuf>,
    pub links: Option<OsString>,
    pub crate_dir: Option<PathBuf>,
    pub include_dir: Option<PathBuf>,
}

impl Crate {
    pub fn print_to_cargo(&self) {
        if let Some(include_prefix) = &self.include_prefix {
            println!("cargo:CXXBRIDGE_PREFIX={}", include_prefix.to_string_lossy());
        }
        if let Some(links) = &self.links {
            println!("cargo:CXXBRIDGE_LINKS={}", links.to_string_lossy());
        }
        if let Some(crate_dir) = &self.crate_dir {
            println!("cargo:CXXBRIDGE_CRATE={}", crate_dir.to_string_lossy());
        }
        if let Some(include_dir) = &self.include_dir {
            println!("cargo:CXXBRIDGE_INCLUDE={}", include_dir.to_string_lossy());
        }
    }
}

pub fn direct_dependencies() -> Vec<Crate> {
    let mut crates: BTreeMap<String, Crate> = BTreeMap::new();

    for (k, v) in env::vars_os() {
        let mut k = k.to_string_lossy().into_owned();
        // Only variables set from a build script of direct dependencies are
        // observable. That's exactly what we want! Your crate needs to declare
        // a direct dependency on the other crate in order to be able to
        // #include its headers.
        //
        // Also, they're only observable if the dependency's manifest contains a
        // `links` key. This is important because Cargo imposes no ordering on
        // the execution of build scripts without a `links` key. When exposing a
        // generated header for the current crate to #include, we need to be
        // sure the dependency's build script has already executed and emitted
        // that generated header.
        //
        // References:
        //   - https://doc.rust-lang.org/cargo/reference/build-scripts.html#the-links-manifest-key
        //   - https://doc.rust-lang.org/cargo/reference/build-script-examples.html#using-another-sys-crate
        if k.starts_with("DEP_") {
            if k.ends_with("_CXXBRIDGE_PREFIX") {
                k.truncate(k.len() - "_CXXBRIDGE_PREFIX".len());
                crates.entry(k).or_default().include_prefix = Some(PathBuf::from(v));
            } else if k.ends_with("_CXXBRIDGE_LINKS") {
                k.truncate(k.len() - "_CXXBRIDGE_LINKS".len());
                crates.entry(k).or_default().links = Some(v);
            } else if k.ends_with("_CXXBRIDGE_CRATE") {
                k.truncate(k.len() - "_CXXBRIDGE_CRATE".len());
                crates.entry(k).or_default().crate_dir = Some(PathBuf::from(v));
            } else if k.ends_with("_CXXBRIDGE_INCLUDE") {
                k.truncate(k.len() - "_CXXBRIDGE_INCLUDE".len());
                crates.entry(k).or_default().include_dir = Some(PathBuf::from(v));
            }
        }
    }

    crates.into_iter().map(|entry| entry.1).collect()
}
