use anyhow::Result;
use std::{
    collections::VecDeque,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct Opts {
    pub verbose: bool,
    pub crate_name: String,
    pub name: String,
    pub lib_dir: PathBuf,
    pub optional: bool,
    pub offline: bool,
    pub no_default_features: bool,
    pub features: Option<Vec<String>>,
    pub path: Option<PathBuf>,
    pub package: Option<String>,
    pub rename: Option<String>,
}

impl Opts {
    pub fn from_args(app: clap::App) -> Self {
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

#[derive(Debug)]
pub struct Workspace {
    target_package_name: String,
    target_is_root_package: bool,
    cargo_toml_file: PathBuf,
    cargo_toml_doc: toml_edit::Document,
}

impl Workspace {
    /// Walks upwards starting from `dir`, trying to find
    /// - a target package (that the dylib should be added to)
    /// - a cargo workspace Cargo.toml file
    ///
    /// If no Cargo.toml workspace file is found will return `None`. If one is
    /// found but no target package, will return an error.
    pub fn find_starting_in_dir(
        dir: impl AsRef<Path>,
        target_package_name: Option<impl AsRef<str>>,
    ) -> Result<Option<Self>> {
        tracing::debug!("trying to find workspace");

        let mut target_package_name = target_package_name.map(|n| n.as_ref().to_string());
        let mut workspace_toml = None;
        let mut target_is_root_package = false;
        let mut dir = dir.as_ref();
        let mut relative_path_to_current_dir = Vec::new();

        loop {
            let cargo_toml = dir.join("Cargo.toml");

            if cargo_toml.exists() {
                let cargo_content = fs::read_to_string(&cargo_toml)?;
                let doc = cargo_content.parse::<toml_edit::Document>()?;
                let is_workspace = doc
                    .get("workspace")
                    .map(|w| w.is_table_like())
                    .unwrap_or(false);

                if is_workspace {}

                if target_package_name.is_none() {
                    if let Some(name) = doc
                        .get("package")
                        .and_then(|p| p.get("name"))
                        .and_then(|n| n.as_str())
                    {
                        tracing::debug!("found target package: {name}");
                        target_package_name = Some(name.to_string());
                        if is_workspace {
                            target_is_root_package = true;
                        }
                    }
                }

                if is_workspace {
                    tracing::debug!("found workspace toml at {cargo_toml:?}");
                    workspace_toml = Some((cargo_toml, doc));
                    break;
                }
            }

            if let Some(parent_dir) = dir.parent() {
                if let Some(dir_name) = dir.file_name() {
                    relative_path_to_current_dir.push(dir_name);
                }
                dir = parent_dir;
            } else {
                break;
            }
        }

        match (workspace_toml, target_package_name) {
            (None, _) => Ok(None),
            (Some(_), None) => Err(anyhow::anyhow!(
                "Found workspace but no target package. Please specify a package with --package."
            )),
            (Some((cargo_toml_file, cargo_toml_doc)), Some(package)) => {
                let workspace = Self {
                    target_package_name: package,
                    target_is_root_package,
                    cargo_toml_doc,
                    cargo_toml_file,
                };
                Ok(if workspace.target_package_is_in_workspace()? {
                    tracing::debug!(
                        "target package {} is in workspace {:?}. Is it a root package? {}",
                        workspace.target_package_name,
                        workspace.cargo_toml_file,
                        workspace.target_is_root_package
                    );
                    Some(workspace)
                } else {
                    None
                })
            }
        }
    }

    fn members(&self) -> Vec<String> {
        if let Some(members) = self
            .cargo_toml_doc
            .get("workspace")
            .and_then(|el| el.get("members"))
            .and_then(|el| el.as_array())
        {
            members
                .into_iter()
                .filter_map(|ea| ea.as_str().map(|s| s.to_string()))
                .collect()
        } else {
            Vec::new()
        }
    }

    fn target_package_is_in_workspace(&self) -> Result<bool> {
        if self.target_is_root_package {
            return Ok(true);
        }

        for member in self.members() {
            let base_dir = self.cargo_toml_file.parent().unwrap();
            let member_toml = base_dir.join(&member).join("Cargo.toml");
            if member_toml.exists() {
                let toml = fs::read_to_string(member_toml)?;
                let doc = toml.parse::<toml_edit::Document>()?;
                let member_is_target = doc
                    .get("package")
                    .and_then(|p| p.get("name"))
                    .and_then(|n| n.as_str())
                    .map(|name| name == self.target_package_name)
                    .unwrap_or(false);
                if member_is_target {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    pub fn relative_path_to_workspace_from(
        &self,
        dir: impl AsRef<Path>,
    ) -> Result<Option<PathBuf>> {
        let dir = dir.as_ref();
        let mut dir = if !dir.is_absolute() {
            dir.canonicalize()?
        } else {
            dir.to_path_buf()
        };
        let workspace_dir = self.cargo_toml_file.parent().unwrap();
        let mut relative = VecDeque::new();

        loop {
            if workspace_dir == dir {
                break;
            }
            if let Some(parent) = dir.parent() {
                relative.push_front(dir.file_name().map(PathBuf::from).unwrap());
                dir = parent.to_path_buf();
            } else {
                return Ok(None);
            }
        }

        if relative.is_empty() {
            return Ok(None);
        }

        Ok(Some(
            relative
                .into_iter()
                .fold(PathBuf::from(""), |path, ea| path.join(ea)),
        ))
    }

    pub fn add_member(&mut self, member: impl AsRef<str>) -> Result<()> {
        if let Some(members) = self
            .cargo_toml_doc
            .get_mut("workspace")
            .and_then(|el| el.get_mut("members"))
            .and_then(|el| el.as_array_mut())
        {
            members.push(member.as_ref());
        }

        fs::write(&self.cargo_toml_file, self.cargo_toml_doc.to_string())?;

        Ok(())
    }
}

pub struct DylibWrapperPackage {
    opts: Opts,
}

impl DylibWrapperPackage {
    pub fn new(opts: Opts) -> Self {
        Self { opts }
    }

    /// Create a new package for wrapping the libray as a dylib
    pub fn cargo_new_lib(&self) -> Result<()> {
        let opts = &self.opts;

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

    /// Add the dependency-to-be-wrapped to the package created by
    /// [`cargo_new_lib`].
    pub fn cargo_add_dependency_to_new_lib(&self) -> Result<()> {
        let opts = &self.opts;

        let mut args = vec!["add", &opts.crate_name];

        if opts.offline {
            args.push("--offline");
        }

        if let Some(features) = self
            .opts
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

    /// Modify the source of the package created by [`cargo_new_lib`] to re-export
    /// the original package and make it `crate-type = ["dylib"]`.
    pub fn modify_dynamic_lib(&self) -> Result<()> {
        let opts = &self.opts;

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
}

pub struct TargetPackage {
    opts: Opts,
}

impl TargetPackage {
    pub fn new(opts: Opts) -> Self {
        Self { opts }
    }

    /// Modify the target package that should make use of the original dependency as
    /// dylib.
    pub fn cargo_add_dynamic_library_to_target_package(&self) -> Result<()> {
        let opts = &self.opts;

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
}
