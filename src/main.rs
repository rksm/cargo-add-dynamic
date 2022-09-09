use std::{fs, io::Write};

use anyhow::Result;
use clap::{App, Arg};

mod opts;
use crate::opts::Opts;

pub const EXIT_CODE_ERROR: i32 = 2;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::builder().parse_lossy("debug")),
        )
        .init();

    let app = App::new("add-dynamic")
        .arg(Arg::new("crate-name").required(true))
        .arg(Arg::new("optional").long("optional"))
        .arg(Arg::new("offline").long("offline"))
        .arg(Arg::new("no-default-features").long("no-default-features"))
        .arg(
            Arg::new("features")
                .long("features")
                .short('F')
                .multiple_values(true)
                .takes_value(true)
                .value_name("FEATURES")
                .required(false),
        )
        .arg(
            Arg::new("path")
                .help("Filesystem path to local crate to add")
                .long("path")
                .required(false),
        )
        .arg(
            Arg::new("name")
                .help("Name of the dynamic library, defaults to ${crate-name}-dynamic")
                .long("name")
                .short('n')
                .required(false),
        )
        .arg(
            Arg::new("package")
                .help("Package to modify")
                .long("package")
                .short('p')
                .value_name("SPEC")
                .required(false),
        );

    let opts = Opts::from_args(app);

    run(opts)?;
    Ok(())
}

fn run(opts: Opts) -> Result<()> {
    cargo_new_lib(&opts)?;
    cargo_add_dependency_to_new_lib(&opts)?;
    modify_dynamic_lib(&opts)?;
    cargo_add_dynamic_library_to_target_package(&opts)?;

    Ok(())
}

fn cargo_new_lib(opts: &Opts) -> Result<()> {
    let args = vec!["new", "--lib", "--name", &opts.name, opts.lib_dir_str()];

    tracing::debug!("running cargo {}", args.join(" "));

    let result = std::process::Command::new("cargo")
        .args(args)
        .spawn()
        .and_then(|mut proc| proc.wait())?;

    if !result.success() {
        let code = result.code().unwrap_or(EXIT_CODE_ERROR);
        std::process::exit(code);
    }

    Ok(())
}

fn cargo_add_dependency_to_new_lib(opts: &Opts) -> Result<()> {
    let mut args = vec!["add", &opts.crate_name];

    if opts.offline {
        args.push("--offline");
    }

    if let Some(features) = opts
        .features
        .as_ref()
        .map(|features| features.iter().map(|ea| ea.as_str()).collect::<Vec<_>>())
    {
        args.push("--features");
        args.extend(features);
    }

    if opts.no_default_features {
        args.push("--no-default-features");
    }

    if let Some(path) = &opts.path {
        args.push("--path");
        args.push(path.to_str().expect("path"));
    }

    tracing::debug!("running cargo {}", args.join(" "));

    let result = std::process::Command::new("cargo")
        .args(args)
        .current_dir(opts.lib_dir_str())
        .spawn()
        .and_then(|mut proc| proc.wait())?;

    if !result.success() {
        let code = result.code().unwrap_or(EXIT_CODE_ERROR);
        std::process::exit(code);
    }
    Ok(())
}

fn modify_dynamic_lib(opts: &Opts) -> Result<()> {
    let cargo_toml = opts.lib_dir.join("Cargo.toml");
    tracing::debug!("Updating {cargo_toml:?}");
    let mut cargo_toml = fs::OpenOptions::new().append(true).open(cargo_toml)?;
    writeln!(cargo_toml, "\n[lib]\ncrate-type = [\"dylib\"]")?;

    let lib_rs = opts.lib_dir.join("src/lib.rs");
    tracing::debug!("Updating {lib_rs:?}");
    let mut lib_rs = fs::OpenOptions::new()
        .truncate(true)
        .write(true)
        .open(lib_rs)?;
    let crate_name = opts.crate_name.replace('-', "_");
    writeln!(lib_rs, "pub use {crate_name}::*;")?;

    Ok(())
}

fn cargo_add_dynamic_library_to_target_package(opts: &Opts) -> Result<()> {
    let mut args = vec![
        "add",
        &opts.name,
        "--rename",
        &opts.crate_name,
        "--path",
        opts.lib_dir_str(),
    ];

    if opts.offline {
        args.push("--offline");
    }

    if opts.optional {
        args.push("--optional");
    }

    if let Some(package) = &opts.package {
        args.push("--package");
        args.push(package);
    }

    tracing::debug!("running cargo {}", args.join(" "));

    let result = std::process::Command::new("cargo")
        .args(args)
        .spawn()
        .and_then(|mut proc| proc.wait())?;

    if !result.success() {
        let code = result.code().unwrap_or(EXIT_CODE_ERROR);
        std::process::exit(code);
    }
    Ok(())
}
