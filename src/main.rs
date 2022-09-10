use std::{fs, io::Write, path::PathBuf};

use anyhow::Result;
use clap::{App, Arg};

#[derive(Debug)]
struct Opts {
    verbose: bool,
    crate_name: String,
    name: String,
    lib_dir: PathBuf,
    optional: bool,
    offline: bool,
    no_default_features: bool,
    features: Option<Vec<String>>,
    path: Option<PathBuf>,
    package: Option<String>,
    rename: Option<String>,
}

impl Opts {
    fn from_args(app: App) -> Self {
        // for cargo invocation... how to do this correctly?
        let mut args = std::env::args().collect::<Vec<_>>();
        if args[1] == "add-dynamic" {
            args.remove(1);
        }

        let args = app.get_matches_from(args);

        let crate_name = args.value_of("crate-name").expect("crate_name").to_string();

        let name = args
            .value_of("name")
            .map(|n| n.to_string())
            .unwrap_or_else(|| format!("{crate_name}-dynamic"));

        let path = args.value_of("path").map(|p| {
            let p = PathBuf::from(p);
            if p.is_relative() {
                std::env::current_dir().expect("cwd").join(p)
            } else {
                p
            }
        });

        let package = args.value_of("package").map(|p| p.to_string());

        let rename = args.value_of("rename").map(|n| n.to_string());

        let lib_dir = args
            .value_of("lib-dir")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(&name));

        let verbose = args.is_present("verbose");

        let optional = args.is_present("optional");

        let offline = args.is_present("offline");

        let no_default_features = args.is_present("no-default-features");

        let features = args
            .values_of("features")
            .map(|features| features.map(|ea| ea.to_string()).collect());

        Self {
            verbose,
            crate_name,
            name,
            lib_dir,
            path,
            optional,
            offline,
            no_default_features,
            package,
            features,
            rename,
        }
    }

    fn lib_dir_str(&self) -> &str {
        self.lib_dir.to_str().unwrap()
    }
}

fn main() {
    let app = App::new("add-dynamic")
        .about("Cargo command similar to `cargo add` that will add a dependency <DEP> as a dynamic library (dylib) crate by creating a new sub-package whose only dependency is the specified <DEP> and whose crate-type is [\"dylib\"].")
        .arg(Arg::new("crate-name")
             .value_name("DEP")
             .required(true))
        .arg(
            Arg::new("optional")
                .help("Mark the dependency as optional. The package name will be exposed as feature of your crate.")
                .long("optional"),
        )
        .arg(
            Arg::new("verbose")
                .help("Additional (debug) logging.")
                .long("verbose")
                .short('v'),
        )
        .arg(Arg::new("offline")
             .help("Run without accessing the network")
             .long("offline"))
        .arg(
            Arg::new("no-default-features")
                .help("Disable the default features")
                .long("no-default-features"),
        )
        .arg(
            Arg::new("features")
                .help("Space or comma separated list of features to activate")
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
                .value_name("PATH")
                .required(false),
        )
        .arg(
            Arg::new("rename")
                .help("Rename the dependency\nExample uses:\n- Depending on multiple versions of a crate\n- Depend on crates with the same name from different registries")
                .long("rename")
                .value_name("NAME")
                .required(false),
        )
        .arg(
            Arg::new("name")
                .help("Name of the dynamic library, defaults to <DEP>-dynamic")
                .long("name")
                .short('n')
                .value_name("NAME")
                .required(false),
        )
        .arg(
            Arg::new("lib-dir")
                .help("Directory for the new sub-package. Defaults to <DEP>-dynamic")
                .long("lib-dir")
                .value_name("DIR")
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

    let level = if opts.verbose { "debug" } else { "info" };

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::builder().parse_lossy(level)),
        )
        .init();

    run(opts).unwrap();
}

fn run(opts: Opts) -> Result<()> {
    cargo_new_lib(&opts)?;
    cargo_add_dependency_to_new_lib(&opts)?;
    modify_dynamic_lib(&opts)?;
    cargo_add_dynamic_library_to_target_package(&opts)?;

    Ok(())
}

fn cargo_new_lib(opts: &Opts) -> Result<()> {
    let mut args = vec!["new", "--lib", "--name", &opts.name, opts.lib_dir_str()];

    tracing::debug!("running cargo {}", args.join(" "));

    let result = std::process::Command::new("cargo")
        .args(args)
        .spawn()
        .and_then(|mut proc| proc.wait())?;

    if !result.success() {
        let code = result.code().unwrap_or(2);
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
        let code = result.code().unwrap_or(2);
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
    let name = opts.rename.as_ref().unwrap_or(&opts.crate_name);

    let mut args = vec![
        "add",
        &opts.name,
        "--rename",
        name,
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
        let code = result.code().unwrap_or(2);
        std::process::exit(code);
    }
    Ok(())
}
