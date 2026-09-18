//! Where the default jurisdiction is remembered.
//!
//! Choosing a reckoning is a once-in-a-lifetime decision for most people, so it
//! should not be a flag typed every morning. The first run writes the choice
//! down; every run after reads it. An explicit flag always wins over the file.

use std::path::PathBuf;

pub const ENV_VAR: &str = "TYPIKON_JURISDICTION";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Source {
    Flag,
    Environment,
    Config,
    Default,
}

pub fn path() -> PathBuf {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let home = std::env::var_os("HOME").unwrap_or_default();
            PathBuf::from(home).join(".config")
        });
    base.join("typikon").join("config.toml")
}

/// The one key this file holds, read without a TOML parser because a
/// dependency for `key = "value"` is a dependency too many.
fn read() -> Option<String> {
    let text = std::fs::read_to_string(path()).ok()?;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else { continue };
        if key.trim() != "jurisdiction" {
            continue;
        }
        let value = value.trim().trim_matches(['"', '\'']);
        if !value.is_empty() {
            return Some(value.to_string());
        }
    }
    None
}

pub fn write(jurisdiction: &str) -> std::io::Result<PathBuf> {
    let p = path();
    if let Some(dir) = p.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(
        &p,
        format!(
            "# Which reckoning typikon uses when no --jurisdiction is given.\n\
             # Run `typikon --list` to see every accepted value.\n\
             jurisdiction = \"{jurisdiction}\"\n"
        ),
    )?;
    Ok(p)
}

/// The environment beats the config file; both beat the built-in default.
pub fn default_jurisdiction() -> (Option<String>, Source) {
    if let Ok(v) = std::env::var(ENV_VAR) {
        if !v.is_empty() {
            return (Some(v), Source::Environment);
        }
    }
    match read() {
        Some(v) => (Some(v), Source::Config),
        None => (None, Source::Default),
    }
}
