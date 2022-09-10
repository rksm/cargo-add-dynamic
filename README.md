# cargo add-dynamic

[![Crates.io](https://img.shields.io/crates/v/cargo-add-dynamic)](https://crates.io/crates/cargo-add-dynamic)

This cargo command allows to wrap dependencies as dylibs.

For why you might want this see [Speeding up incremental Rust compilation with dylibs](https://robert.kra.hn/posts/2022-09-09-speeding-up-incremental-rust-compilation-with-dylibs/).


## Installation

```shell
cargo install cargo-add-dynamic
```

## Example

To add a new dependency as a dylib to the current project run for example

```shell
cargo add-dynamic polars --features csv-file,lazy,list,describe,rows,fmt,strings,temporal
```

This will create a sub-package `polars-dynamic` with the following content.

`polars-dynamic/Cargo.toml`

```toml
[package]
name = "polars-dynamic"
version = "0.1.0"
edition = "2021"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]
polars = { version = "0.23.2", features = ["csv-file", "lazy", "list", "describe", "rows", "fmt", "strings", "temporal"] }

[lib]
crate-type = ["dylib"]
```

`polars-dynamic/src/lib.rs`

```rust
pub use polars::*;
```

And it will add `polars = { version = "0.1.0", path = "polars-dynamic", package = "polars-dynamic" }` to the dependencies of the current package.


## Usage

```
add-dynamic 
Cargo command similar to `cargo add` that will add a dependency <DEP> as a dynamic library (dylib)
crate by creating a new sub-package whose only dependency is the specified <DEP> and whose
crate-type is ["dylib"].

USAGE:
    cargo-add-dynamic [OPTIONS] <DEP>

ARGS:
    <DEP>    

OPTIONS:
    -F, --features <FEATURES>...    Space or comma separated list of features to activate
    -h, --help                      Print help information
        --lib-dir <DIR>             Directory for the new sub-package. Defaults to <DEP>-dynamic
    -n, --name <NAME>               Name of the dynamic library, defaults to <DEP>-dynamic
        --no-default-features       Disable the default features
        --offline                   Run without accessing the network
        --optional                  Mark the dependency as optional. The package name will be
                                    exposed as feature of your crate.
    -p, --package <SPEC>            Package to modify
        --path <PATH>               Filesystem path to local crate to add
        --rename <NAME>             Rename the dependency
                                    Example uses:
                                    - Depending on multiple versions of a crate
                                    - Depend on crates with the same name from different registries
    -v, --verbose                   Additional (debug) logging.
```
