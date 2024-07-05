use anyhow::bail;
use clap::Parser;
use std::io::prelude::*;
use std::path::PathBuf;
use toml_edit::{DocumentMut, Value};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The `Cargo.toml` file to sanitize
    #[arg(long, short = 'i', required = true)]
    input_file: PathBuf,
    /// The `Cargo.toml` file to sanitize
    #[arg(long, short = 'o')]
    output: Option<PathBuf>,
}

const KEYS_TO_REMOVE: [&str; 5] = ["git", "tag", "branch", "rev", "path"];

fn sanitize_dependency_entry(ent: &mut toml_edit::Value) -> anyhow::Result<()> {
    match ent {
        Value::String(_s) => { /* do nothing */ }
        Value::Integer(_i) => { /* do nothing */ }
        Value::Float(_f) => { /* do nothing */ }
        Value::Boolean(_b) => { /* do nothing */ }
        Value::Datetime(_d) => { /* do nothing */ }
        Value::Array(_a) => { /* do nothing */ }
        Value::InlineTable(t) => {
            for k in KEYS_TO_REMOVE {
                t.remove(k);
            }
        }
    }
    Ok(())
}

fn sanitize(doc: &mut DocumentMut) -> anyhow::Result<()> {
    if let Some(ref mut deps) = &mut doc["dependencies"].as_table_like_mut() {
        for (_k, item) in deps.iter_mut() {
            match item {
                toml_edit::Item::Table(_t) => {
                    bail!("unexpected format of dependency table");
                }
                toml_edit::Item::Value(v) => {
                    sanitize_dependency_entry(v)?;
                }
                toml_edit::Item::ArrayOfTables(_aot) => {
                    bail!("unexpected format of dependency table");
                }
                _ => {}
            };
        }
    } else {
        bail!("The [dependencies] table is absent; no changes will be made.");
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let orig_str = std::fs::read_to_string(cli.input_file)?;
    let mut toml_contents = orig_str.parse::<DocumentMut>()?;
    sanitize(&mut toml_contents)?;

    if let Some(out_path) = cli.output {
        let mut ofile = std::fs::File::create(out_path)?;
        ofile.write_all(toml_contents.to_string().as_bytes())?;
    } else {
        println!("{}", toml_contents);
    }
    Ok(())
}
