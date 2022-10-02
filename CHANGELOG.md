# Changelog

This package tries hard to adhere to [semver](https://semver.org/).

## [0.1.2]
### Changed
- support for workspaces:

When `cargo add-dynamic` is invoked, we will try to find a workspace toml file
starting from cwd. We will check if the target package (the package the dylib
should be added to) is a workspace member and if this is the case we will add
the dylib package as a member as well.

## [0.1.1]
### Added
- support for `--rename`, `--lib-dir`, `--verbose` flags

## [0.1.0]
### Added
- `cargo add-dynamic` command
