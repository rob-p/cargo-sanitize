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

fn get_crate_version(ent: &mut toml_edit::Value) -> anyhow::Result<String> {
    let mut did_remove = false;
    match ent {
        Value::String(s) => Ok(s.to_string()),
        Value::Integer(i) => Ok(i.to_string()),
        Value::Float(f) => Ok(f.to_string()),
        Value::Boolean(_b) => Err(anyhow!("cannot convert boolean to version string")),
        Value::Datetime(_d) => Err(anyhow!("cannot convert datetime to version string")),
        Value::Array(_a) => Err(anyhow!("cannont convert array to version string")),
        Value::InlineTable(t) => Ok(format!(
            "\"{}\"",
            t.get("version").unwrap().as_str().unwrap()
        )),
    }
}

#[cfg(feature = "verify_crates")]
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

#[cfg(feature = "verify_crates")]
fn get_crate_info_as_json(
    key: String,
    agent: &ureq::Agent,
) -> anyhow::Result<Vec<serde_json::Value>> {
    let mut body_res = String::from("[");
    body_res.push_str(
        &agent
            .get(&format!("https://index.crates.io/{}", crate_key(key)))
            .call()?
            .into_string()?,
    );
    body_res = body_res.replace("\n", ",\n");
    body_res.pop();
    body_res.pop();
    body_res.push(']');
    match serde_json::from_str::<serde_json::Value>(&body_res) {
        Ok(serde_json::Value::Array(a)) => Ok(a),
        Ok(v) => Err(anyhow!("unexpected JSON types")),
        Err(e) => Err(anyhow!("Invalid JSON : {:?}", e)),
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

                    #[cfg(feature = "verify_crates")]
                    if did_remove {
                        let ver = get_crate_version(v)?;
                        let a = get_crate_info_as_json(_k.to_string(), &agent)?;
                        for x in a.iter() {
                            let fetched_ver = x.get("vers").unwrap().to_string();
                            if ver == fetched_ver {
                                eprintln!(
                                    "observed_version: {}, fetched_version {}",
                                    ver, fetched_ver
                                );
                            } else {
                                eprintln!(
                                    "observed_version: [{}] != fetched_version [{}]",
                                    ver, fetched_ver
                                );
                            }
                        }
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
