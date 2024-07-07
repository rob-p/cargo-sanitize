use anyhow::{anyhow, bail};
use clap::Parser;
use std::io::prelude::*;
use std::path::PathBuf;
use toml_edit::{DocumentMut, Value};

#[cfg(feature = "verify_crates")]
use std::time::Duration;
#[cfg(feature = "verify_crates")]
use ureq::{Agent, AgentBuilder};

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

fn sanitize_dependency_entry(ent: &mut toml_edit::Value) -> anyhow::Result<bool> {
    let mut did_remove = false;
    match ent {
        Value::String(_s) => { /* do nothing */ }
        Value::Integer(_i) => { /* do nothing */ }
        Value::Float(_f) => { /* do nothing */ }
        Value::Boolean(_b) => { /* do nothing */ }
        Value::Datetime(_d) => { /* do nothing */ }
        Value::Array(_a) => { /* do nothing */ }
        Value::InlineTable(t) => {
            for k in KEYS_TO_REMOVE {
                if let Some(_v) = t.remove(k) {
                    did_remove = true;
                }
            }
        }
    }
    Ok(did_remove)
}

fn crate_key(s: String) -> String {
    let mut pref = String::new();
    match s.len() {
        1 => format!("1/{}", s),
        2 => format!("2/{}", s),
        3 => format!("3/{}/{}", &s[0..1], s),
        4.. => format!("{}/{}/{}", &s[0..2], &s[2..4], s),
        _ => format!("ERROR"),
    }
}

fn sanitize(doc: &mut DocumentMut) -> anyhow::Result<()> {
    #[cfg(feature = "verify_crates")]
    // Instantiate the client.
    let agent = ureq::AgentBuilder::new()
        .timeout_read(Duration::from_secs(5))
        .timeout_write(Duration::from_secs(5))
        .build();

    if let Some(ref mut deps) = &mut doc["dependencies"].as_table_like_mut() {
        for (_k, item) in deps.iter_mut() {
            match item {
                toml_edit::Item::Table(_t) => {
                    bail!("unexpected format of dependency table");
                }
                toml_edit::Item::Value(v) => {
                    let did_remove = sanitize_dependency_entry(v)?;
                    eprintln!("v : {:?}", v);
                    if did_remove {
                        eprintln!("{:#?}", _k.get());
                    }
                    #[cfg(feature = "verify_crates")]
                    if did_remove {
                        let body: String = agent
                            .get(&format!(
                                "https://index.crates.io/{}",
                                crate_key(_k.to_string())
                            ))
                            .call()?
                            .into_string()?;
                        /*
                        let r = client.get_crate(_k.get()).map_err(|e| {
                            anyhow!(
                                "could not verify the crate {} in the registry; error {:?}",
                                _k.get(),
                                e
                            )
                        })?;
                        */
                        eprintln!("body {}", body);
                        //eprintln!("verified crate {}", r.crate_data.id);
                    }
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
