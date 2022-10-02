use anyhow::Result;
use cargo_add_dynamic::{DylibWrapperPackage, Opts, TargetPackage, Workspace};
use clap::{App, Arg};

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
    // optional: update workspace so that it contains the new dylib as a member
    let cwd = std::env::current_dir()?;
    if let Some(mut workspace) = Workspace::find_starting_in_dir(&cwd, opts.package.as_ref())? {
        let lib_dir = if let Some(path_to_cwd) = workspace.relative_path_to_workspace_from(cwd)? {
            path_to_cwd.join(&opts.lib_dir)
        } else {
            opts.lib_dir.clone()
        };
        workspace.add_member(lib_dir.to_string_lossy())?;
    }

    // create the dylib package
    let dylib_package = DylibWrapperPackage::new(opts.clone());
    dylib_package.cargo_new_lib()?;
    dylib_package.cargo_add_dependency_to_new_lib()?;
    dylib_package.modify_dynamic_lib()?;

    // add the dylib package to the target package
    let target_package = TargetPackage::new(opts);
    target_package.cargo_add_dynamic_library_to_target_package()?;

    Ok(())
}
