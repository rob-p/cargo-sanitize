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


## Differences from related tools

The functionality provided by `cargo-sanitize` is very tightly-scoped.  Related (and more general) functionality is provided by 
the [`cargo publish`](https://doc.rust-lang.org/cargo/commands/cargo-publish.html) and [`cargo package`](https://doc.rust-lang.org/cargo/commands/cargo-package.html) comamnds. 
Specifically, the sole purpose of `cargo-sanitize` is to rewrite your `Cargo.toml` file so that no dependencies absent from the downstream registry (i.e. `crates.io`) are present.  
Therefore:

  - It does *not* attempt to build your project. If your requirements are not properly constrained by the stated version of the dependenices it is 
  possible that moving from the path dependency to the registry dependency could break the build.  **Note**: `cargo-sanitize` _can_
  "nominally" verify version constraints, and as long as your application abides by these, this should not be a problem.

  - It does *not* attempt to "normalize" or re-organize your `Cargo.toml` file in any way.  Specifically, `cargo-sanitize` relies
  on the [`toml_edit`](https://crates.io/crates/toml_edit) crate, and so, apart from the removal of path dependencies, it attempts to
  retain the original formatting of the `Cargo.toml` file as much as possible. 

  - It does *not* inspect or touch any other files in your project. It will not attempt to create a package of your project (either as a tarball or a `.crate` file).
  It will not read or write other files in your project directory or workspace.  It can be used to *only* sanitize the `Cargo.toml` file --- of course, this 
  means it is the only thing that it *can* do.

  - It *can* work in a completely offline mode; rewriting the `Cargo.toml` file without any access to the network.  Of course, in this case, there is 
  no validation performed that the rewritten dependencies exist in the downstream registry, and you are therefore entirely responsible for guaranteeing
  that yourself.

If you are looking to do more than to *simply* sanitize your `Cargo.toml` file, and are looking for more robust or fully-fledged functionality, 
then you are likely interested in the the [`cargo publish`](https://doc.rust-lang.org/cargo/commands/cargo-publish.html) or 
[`cargo package`](https://doc.rust-lang.org/cargo/commands/cargo-package.html) comamnds. 
