# Welcome to the documentation for cargo-sanitize

Documentation for the `cargo-sanitize` program 

## Purpose

The `cargo-sanitize` program removes custom paths / sources from dependencies in your `Cargo.toml` file 
(i.e. those pointing to `git` repositories or local `paths`).  It is intended to provide a _super lightweight_ 
version of the transformation that `cargo publish` does to your `Cargo.toml` file before publishing to 
`crates.io`.

## Use

To use `cargo-sanitize`, you simply pass the executable your current `Cargo.toml` file.  If you provide no 
output path, the new `Cargo.toml` contents will be written directly to `stdout`.  Alternatively, you can 
provide the new file path with the `-o` option.  A standard invocation might look like:

```
cargo-sanitize -i Cargo.toml.orig > Cargo.toml
```

Here, we are assuming that you have already copied your "source" `Cargo.toml` to the file `Cargo.toml.orig`, and 
this command will then write the sanitized version to the path `Cargo.toml`.

## Features

`cargo-sanitize` provides the optional `validate_crates` feature.  This feature will query the cargo registry for
the dependencies listed in your `Cargo.toml`, and will succeed only if there is a compatible version of each 
declared dependency present in the registry (compatibility is determined via the [semver crate](https://crates.io/crates/semver)).

This can ensure that, e.g. if you are relying on a local path at a particular version, that version (or a compatible one) 
is also available downstream in the official registry.  

If you install `cargo-sanitize` with this feature, then you can use the `--validate-type` option to validate either 
(1) all of the dependencies (by passing `--validate-type all`) or (2) only the dependencies that are rewritten 
(by passing `--validate-type rewritten`).  **Note**: Even if you enable this feature, the default value for the 
`--validate-type` option is `none`, which will not perform validation.  This behavior is designed so that `cargo-sanitize`
need not require network access by default to sanitize the input `Cargo.toml` file.

