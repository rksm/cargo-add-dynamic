use anyhow::Result;
use clap::{App, Arg};

mod opts;
use crate::opts::Opts;

mod consts;

mod core;
use crate::core::run;

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
