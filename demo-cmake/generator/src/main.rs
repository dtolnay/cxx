use clap::{App, Arg, SubCommand};

use semver::Version;
use std::fs::{create_dir_all, File};
use std::io::{stdout, Write};
use std::path::Path;
use std::rc::Rc;

mod platform;
mod target;

const MANIFEST_PATH: &str = "manifest-path";
const OUT_FILE: &str = "out-file";
const CONFIGURATION_TYPE: &str = "configuration-type";
const CONFIGURATION_TYPES: &str = "configuration-types";
const CONFIGURATION_ROOT: &str = "configuration-root";
const TARGET: &str = "target";
const CARGO_VERSION: &str = "cargo-version";

const PRINT_ROOT: &str = "print-root";
const GEN_CMAKE: &str = "gen-cmake";

fn config_type_target_folder(config_type: Option<&str>) -> &'static str {
    match config_type {
        Some("Debug") | None => "debug",
        Some("Release") | Some("RelWithDebInfo") | Some("MinSizeRel") => "release",
        Some(config_type) => panic!("Unknown config_type {}!", config_type),
    }
}

fn main() -> Result<(), Box<std::error::Error>> {
    let matches = App::new("CMake Generator for Cargo")
        .version("0.1")
        .author("Andrew Gaspar <andrew.gaspar@outlook.com>")
        .about("Generates CMake files for Cargo projects")
        .arg(
            Arg::with_name(MANIFEST_PATH)
                .long("manifest-path")
                .value_name("Cargo.toml")
                .help("Specifies the target Cargo project")
                .takes_value(true),
        )
        .subcommand(SubCommand::with_name(PRINT_ROOT))
        .subcommand(
            SubCommand::with_name(GEN_CMAKE)
                .arg(
                    Arg::with_name(CONFIGURATION_ROOT)
                        .long("configuration-root")
                        .value_name("DIRECTORY")
                        .takes_value(true)
                        .help(
                            "Specifies a root directory for configuration folders. E.g. Win32 \
                             in VS Generator.",
                        ),
                )
                .arg(
                    Arg::with_name(CONFIGURATION_TYPE)
                        .long("configuration-type")
                        .value_name("type")
                        .takes_value(true)
                        .conflicts_with(CONFIGURATION_TYPES)
                        .help(
                            "Specifies the configuration type to use in a single configuration \
                             environment.",
                        ),
                )
                .arg(
                    Arg::with_name(CONFIGURATION_TYPES)
                        .long("configuration-types")
                        .value_name("types")
                        .takes_value(true)
                        .multiple(true)
                        .require_delimiter(true)
                        .conflicts_with(CONFIGURATION_TYPE)
                        .help(
                            "Specifies the configuration types to use in a multi-configuration \
                             environment.",
                        ),
                )
                .arg(
                    Arg::with_name(TARGET)
                        .long("target")
                        .value_name("triple")
                        .takes_value(true)
                        .required(true)
                        .help("The build target being used."),
                )
                .arg(
                    Arg::with_name(CARGO_VERSION)
                        .long(CARGO_VERSION)
                        .value_name("version")
                        .takes_value(true)
                        .required(true)
                        .help("Version of target cargo"),
                )
                .arg(
                    Arg::with_name(OUT_FILE)
                        .short("o")
                        .long("out-file")
                        .value_name("FILE")
                        .help("Output CMake file name. Defaults to stdout.")
                        .takes_value(true),
                ),
        )
        .get_matches();

    let mut cmd = cargo_metadata::MetadataCommand::new();

    if let Some(manifest_path) = matches.value_of(MANIFEST_PATH) {
        cmd.manifest_path(Path::new(manifest_path));
    }

    let metadata = cmd.exec().unwrap();

    if let Some(_) = matches.subcommand_matches(PRINT_ROOT) {
        println!("{}", metadata.workspace_root.to_str().unwrap());
        std::process::exit(0);
    }

    let matches = matches.subcommand_matches(GEN_CMAKE).unwrap();

    let cargo_version = Version::parse(matches.value_of(CARGO_VERSION).unwrap())
        .expect("cargo-version must be a semver-compatible version!");

    let cargo_target = matches.value_of(TARGET).and_then(platforms::find).cloned();

    if cargo_target.is_none() {
        println!("WARNING: The target was not recognized.");
    }

    let cargo_platform = platform::Platform::from_rust_version_target(cargo_version, cargo_target);

    let mut out_file: Box<Write> = if let Some(path) = matches.value_of(OUT_FILE) {
        let path = Path::new(path);
        if let Some(parent) = path.parent() {
            create_dir_all(parent).expect("Failed to create directory!");
        }
        let file = File::create(path).expect("Unable to open out-file!");
        Box::new(file)
    } else {
        Box::new(stdout())
    };

    writeln!(
        out_file,
        "\
cmake_minimum_required (VERSION 3.10)
"
    )?;

    let config_root = Path::new(matches.value_of(CONFIGURATION_ROOT).unwrap_or("."));

    let mut config_folders = Vec::new();
    if let Some(config_types) = matches.values_of(CONFIGURATION_TYPES) {
        for config_type in config_types {
            let config_folder = config_root.join(config_type);
            assert!(
                config_folder.join(".cargo/config").exists(),
                "Target config_folder '{}' must contain a '.cargo/config'.",
                config_folder.display()
            );
            config_folders.push((Some(config_type), config_folder));
        }
    } else {
        let config_type = matches.value_of(CONFIGURATION_TYPE);
        let config_folder = config_root;
        assert!(
            config_folder.join(".cargo/config").exists(),
            "Target config_folder '{}' must contain a '.cargo/config'.",
            config_folder.display()
        );
        config_folders.push((config_type, config_folder.to_path_buf()));
    }

    let targets: Vec<_> = metadata
        .packages
        .iter()
        .filter(|p| metadata.workspace_members.contains(&p.id))
        .cloned()
        .map(Rc::new)
        .flat_map(|package| {
            let package2 = package.clone();
            package
                .targets
                .clone()
                .into_iter()
                .filter_map(move |t| target::CargoTarget::from_metadata(package2.clone(), t))
        })
        .collect();

    for target in &targets {
        target
            .emit_cmake_target(&mut out_file, &cargo_platform)
            .unwrap();
    }

    writeln!(out_file)?;

    let metadata_manifest_path = Path::new(&metadata.workspace_root).join("Cargo.toml");

    for (config_type, config_folder) in config_folders {
        let current_dir = std::env::current_dir().expect("Could not get current directory!");
        std::env::set_current_dir(config_folder)
            .expect("Could not change directory to the Config directory!");

        let mut local_metadata_cmd = cargo_metadata::MetadataCommand::new();
        local_metadata_cmd.manifest_path(Path::new(&metadata_manifest_path));

        // Re-gathering the cargo metadata from here gets us a target_directory scoped to the
        // configuration type.
        let local_metadata = local_metadata_cmd
            .exec()
            .expect("Could not open Crate specific metadata!");

        let build_path = Path::new(&local_metadata.target_directory)
            .join(matches.value_of(TARGET).unwrap_or(""))
            .join(config_type_target_folder(config_type));

        for target in &targets {
            target.emit_cmake_config_info(
                &mut out_file,
                &cargo_platform,
                &build_path,
                &config_type,
            )?;
        }

        std::env::set_current_dir(current_dir)
            .expect("Could not return to the build root directory!")
    }

    Ok(())
}
