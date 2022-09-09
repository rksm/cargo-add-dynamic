use std::{fs, io::Write, path::PathBuf};

use anyhow::Result;
use clap::{App, Arg};

#[derive(Default, Debug)]
struct Opts {
    crate_name: String,
    name: String,
    lib_dir: PathBuf,
    optional: bool,
    offline: bool,
    no_default_features: bool,
    features: Option<Vec<String>>,
    path: Option<PathBuf>,
    package: Option<String>,
}

impl Opts {
    fn from_args(app: App) -> Self {
        let args = app.get_matches();

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

        let lib_dir = PathBuf::from(&name);
        let optional = args.is_present("optional");
        let offline = args.is_present("offline");
        let no_default_features = args.is_present("no-default-features");

        Self {
            crate_name,
            name,
            lib_dir,
            path,
            optional,
            offline,
            no_default_features,
            package,
            ..Default::default()
        }
    }

    fn lib_dir_str(&self) -> &str {
        self.lib_dir.to_str().unwrap()
    }
}

fn main() {
    tracing_subscriber::fmt::init();

    let app = App::new("cargo-add-dynamic")
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

    run(opts).unwrap();
    // if let Err(err) = run(opts) {
    //     eprintln!("Error creating dynamic library: {err}");
    //     std::process::exit(1);
    // }
}

fn run(opts: Opts) -> Result<()> {
    dbg!(&opts);

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
    let mut cargo_toml = fs::OpenOptions::new()
        .append(true)
        .open(opts.lib_dir.join("Cargo.toml"))?;
    writeln!(cargo_toml, "\n[lib]\ncrate-type = [\"dylib\"]")?;

    let mut lib_rs = fs::OpenOptions::new()
        .truncate(true)
        .write(true)
        .open(opts.lib_dir.join("src/lib.rs"))?;
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
        let code = result.code().unwrap_or(2);
        std::process::exit(code);
    }
    Ok(())
}
