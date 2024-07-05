# cargo-sanitize

## Purpose

The `cargo-sanitize` program removes custom paths / sources from dependencies in your `Cargo.toml` file 
(i.e. those pointing to `git` repositories or local `paths`).  It is intended to provide a _super lightweight_ 
version of the transformation that `cargo publish` does to your `Cargo.toml` file before publishing to 
`crates.io`.  For more information on `cargo-sanitize`, please refer to [the docs](https://rob-p.github.io/cargo-sanitize/).
