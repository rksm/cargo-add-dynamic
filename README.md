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

USAGE:
    cargo-add-dynamic [OPTIONS] <crate-name>

ARGS:
    <crate-name>    

OPTIONS:
    -F, --features <FEATURES>...    
    -h, --help                      Print help information
    -n, --name                      Name of the dynamic library, defaults to ${crate-name}-dynamic
        --no-default-features       
        --offline                   
        --optional                  
    -p, --package <SPEC>            Package to modify
        --path                      Filesystem path to local crate to add
```
