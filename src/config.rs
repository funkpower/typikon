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

/// Everything the file can hold. All three axes, because remembering only the
/// jurisdiction meant a terminal and a desktop widget on the same machine
/// could disagree about the same day.
#[derive(Default)]
pub struct Saved {
    pub jurisdiction: Option<String>,
    pub tradition: Option<String>,
    /// True when the day should be shown at the Typikon's discipline rather
    /// than at the jurisdiction's own particular law.
    pub typikon: bool,
}

/// Read without a TOML parser, because a dependency for `key = "value"` is a
/// dependency too many.
fn read() -> Saved {
    let mut out = Saved::default();
    let Ok(text) = std::fs::read_to_string(path()) else { return out };
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else { continue };
        let value = value.trim().trim_matches(['"', '\'']);
        if value.is_empty() {
            continue;
        }
        match key.trim() {
            "jurisdiction" => out.jurisdiction = Some(value.to_string()),
            "tradition" => out.tradition = Some(value.to_string()),
            "typikon" => out.typikon = value == "true",
            _ => {}
        }
    }
    out
}

pub fn write(saved: &Saved) -> std::io::Result<PathBuf> {
    let p = path();
    if let Some(dir) = p.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let mut text = String::from(
        "# What typikon assumes when the flags are not given.\n\
         # Run `typikon --list` to see every accepted jurisdiction.\n",
    );
    if let Some(j) = &saved.jurisdiction {
        text.push_str(&format!("jurisdiction = \"{j}\"\n"));
    }
    if let Some(t) = &saved.tradition {
        text.push_str(&format!("tradition = \"{t}\"\n"));
    }
    if saved.typikon {
        text.push_str(
            "\n# Show the day at the Typikon's full discipline rather than at the\n\
             # minimum this church's particular law requires. Obligations are still\n\
             # reported either way.\n\
             typikon = true\n",
        );
    }
    std::fs::write(&p, text)?;
    Ok(p)
}

pub fn saved() -> Saved {
    read()
}

/// The environment beats the config file; both beat the built-in default.
pub fn default_jurisdiction() -> (Option<String>, Source) {
    if let Ok(v) = std::env::var(ENV_VAR) {
        if !v.is_empty() {
            return (Some(v), Source::Environment);
        }
    }
    match read().jurisdiction {
        Some(v) => (Some(v), Source::Config),
        None => (None, Source::Default),
    }
}
