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
cargo-sanitize -i Cargo.orig.toml > Cargo.toml
```

Here, we are assuming that you have already copied your "source" `Cargo.toml` to the file `Cargo.orig.toml`, and 
this command will then write the sanitized version to the path `Cargo.toml`.

