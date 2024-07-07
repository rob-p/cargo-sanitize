use anyhow::bail;
use clap::{Parser, ValueEnum};
use std::fmt;
use std::io::prelude::*;
use std::path::PathBuf;
use toml_edit::{DocumentMut, Value};

#[cfg(feature = "verify_crates")]
use anyhow::anyhow;
#[cfg(feature = "verify_crates")]
use semver::{Version, VersionReq};
#[cfg(feature = "verify_crates")]
use std::time::Duration;
#[cfg(feature = "verify_crates")]
use ureq::AgentBuilder;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    /// Don't validate any crates
    None,
    /// Only validate re-written dependencies
    Rewritten,
    /// Validate all dependencies
    All,
}

impl Default for Mode {
    fn default() -> Self {
        Self::None
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Rewritten => write!(f, "rewritten"),
            Self::All => write!(f, "all"),
        }
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The `Cargo.toml` file to sanitize
    #[arg(long, short = 'i', required = true)]
    input_file: PathBuf,
    /// The `Cargo.toml` file to sanitize
    #[arg(long, short = 'o')]
    output: Option<PathBuf>,
    /// Print relevant messages to stderr
    #[arg(long, short = 'v')]
    verbose: bool,
    /// Validate the existence of all dependencies on crates.io
    #[arg(long, short = 't', default_value_t = Mode::None, value_parser = clap::builder::EnumValueParser::<Mode>::new())]
    validate_type: Mode,
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

#[cfg(feature = "verify_crates")]
fn get_crate_version(ent: &mut toml_edit::Value) -> anyhow::Result<String> {
    let ver_str = match ent {
        Value::String(s) => Ok(s.to_string()),
        Value::Integer(i) => Ok(i.to_string()),
        Value::Float(f) => Ok(f.to_string()),
        Value::InlineTable(t) => Ok(String::from(
            t.get("version")
                .unwrap_or(&toml_edit::Value::from("COULD NOT FIND VERSION"))
                .as_str()
                .unwrap(),
        )),
        Value::Boolean(_b) => Err(anyhow!("cannot convert boolean to version string")),
        Value::Datetime(_d) => Err(anyhow!("cannot convert datetime to version string")),
        Value::Array(_a) => Err(anyhow!("cannont convert array to version string")),
    };

    match ver_str {
        Ok(s) => Ok(s.replace('"', "")),
        Err(e) => Err(e),
    }
}

#[cfg(feature = "verify_crates")]
fn crate_key(s: String) -> anyhow::Result<String> {
    match s.len() {
        1 => Ok(format!("1/{}", s)),
        2 => Ok(format!("2/{}", s)),
        3 => Ok(format!("3/{}/{}", &s[0..1], s)),
        4.. => Ok(format!("{}/{}/{}", &s[0..2], &s[2..4], s)),
        _ => Err(anyhow!("crate name [{}] must have length > 0", s)),
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
            .get(&format!("https://index.crates.io/{}", crate_key(key)?))
            .call()?
            .into_string()?,
    );
    body_res = body_res.replace('\n', ",\n");
    body_res.pop();
    body_res.pop();
    body_res.push(']');
    match serde_json::from_str::<serde_json::Value>(&body_res) {
        Ok(serde_json::Value::Array(a)) => Ok(a),
        Ok(_v) => Err(anyhow!("unexpected JSON types")),
        Err(e) => Err(anyhow!("Invalid JSON : {:?}", e)),
    }
}

#[allow(unused_variables)]
fn sanitize(doc: &mut DocumentMut, validate_type: Mode) -> anyhow::Result<()> {
    #[cfg(feature = "verify_crates")]
    // Instantiate the client.
    let agent = AgentBuilder::new()
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
                    #[allow(unused_variables)]
                    let did_remove = sanitize_dependency_entry(v)?;

                    #[cfg(feature = "verify_crates")]
                    if validate_type == Mode::All
                        || (did_remove && (validate_type == Mode::Rewritten))
                    {
                        let ver = get_crate_version(v)?;
                        let req = VersionReq::parse(&ver).map_err(|e| {
                            anyhow!("could not parse version {} for crate {}!", ver, _k)
                        })?;
                        if let Ok(a) = get_crate_info_as_json(_k.to_string(), &agent) {
                            for x in a.iter() {
                                let fetched_ver_str =
                                    x.get("vers").unwrap().to_string().replace('"', "");
                                let fetched_ver =
                                    Version::parse(&fetched_ver_str).map_err(|e| {
                                        anyhow!(
                                            "could not parse fetched version {} for crate {}!",
                                            fetched_ver_str,
                                            _k
                                        )
                                    })?;
                                if req.matches(&fetched_ver) {
                                    log::info!(
                                        "Verified crate [{}] : observed_version: {} = fetched_version {}",
                                        _k, ver, fetched_ver
                                    );
                                    break;
                                }
                            }
                        } else {
                            bail!("ERROR :: cannot find crate {} on crates.io index!", _k);
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

    if cli.verbose {
        simple_logger::init_with_level(log::Level::Info)?;
    } else {
        simple_logger::init_with_level(log::Level::Warn)?;
    }

    let orig_str = std::fs::read_to_string(cli.input_file)?;
    let mut toml_contents = orig_str.parse::<DocumentMut>()?;

    #[cfg(not(feature = "verify_crates"))]
    match cli.validate_type {
        Mode::All | Mode::Rewritten => {
            bail!("cannot validate crates without the `validate` feature.")
        }
        Mode::None => {}
    }

    sanitize(&mut toml_contents, cli.validate_type)?;

    if let Some(out_path) = cli.output {
        let mut ofile = std::fs::File::create(out_path)?;
        ofile.write_all(toml_contents.to_string().as_bytes())?;
    } else {
        println!("{}", toml_contents);
    }
    Ok(())
}
