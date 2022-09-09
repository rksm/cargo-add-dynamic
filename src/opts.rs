use clap::App;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Opts {
    pub crate_name: String,
    pub name: String,
    pub lib_dir: PathBuf,
    pub optional: bool,
    pub offline: bool,
    pub no_default_features: bool,
    pub features: Option<Vec<String>>,
    pub path: Option<PathBuf>,
    pub package: Option<String>,
}

impl Opts {
    pub fn from_args(app: App) -> Self {
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

        let lib_dir = PathBuf::from(&name);
        let optional = args.is_present("optional");
        let offline = args.is_present("offline");
        let no_default_features = args.is_present("no-default-features");

        let features = args
            .values_of("features")
            .map(|features| features.map(|ea| ea.to_string()).collect());

        Self {
            crate_name,
            name,
            lib_dir,
            path,
            optional,
            offline,
            no_default_features,
            package,
            features,
        }
    }

    pub fn lib_dir_str(&self) -> &str {
        self.lib_dir.to_str().unwrap()
    }
}
