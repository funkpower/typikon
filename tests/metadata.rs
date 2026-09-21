//! The packaging metadata, which lives in three files and is published to two
//! registries.
//!
//! `Cargo.toml` describes the crate, `pyproject.toml` is what maturin puts on
//! PyPI, and `npm/package.json.in` is what npm shows. Nothing makes them agree
//! and nothing notices when they stop: a description edited in one of the three
//! reaches one registry and not the other, and the wrong one is then frozen
//! into a release, because neither registry lets published metadata be edited.
//! That has happened once already.
//!
//! The two READMEs open with the same sentence, because that paragraph is the
//! first thing shown on both registry pages and is what most people will read
//! instead of the description field.

use std::fs;

/// The value of a `key = "…"` line, from the first section that has one.
fn toml_string(path: &str, key: &str) -> String {
    let text = fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    for line in text.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix(key) else { continue };
        let Some(rest) = rest.trim_start().strip_prefix('=') else { continue };
        let rest = rest.trim();
        if let Some(v) = rest.strip_prefix('"').and_then(|v| v.strip_suffix('"')) {
            return v.to_string();
        }
    }
    panic!("{path}: no `{key} = \"…\"` line");
}

/// The value of a `"key": "…"` line. Enough for a hand-written file with no
/// escapes in it, and the test below fails loudly rather than quietly if that
/// ever stops being true.
fn json_string(path: &str, key: &str) -> String {
    let text = fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let needle = format!("\"{key}\":");
    for line in text.lines() {
        let Some(rest) = line.trim().strip_prefix(&needle) else { continue };
        let rest = rest.trim().trim_end_matches(',').trim();
        if let Some(v) = rest.strip_prefix('"').and_then(|v| v.strip_suffix('"')) {
            assert!(!v.contains('\\'), "{path}: {key} has escapes this reader cannot handle");
            return v.to_string();
        }
    }
    panic!("{path}: no `\"{key}\": \"…\"` line");
}

#[test]
fn the_three_descriptions_are_the_same_sentence() {
    let cargo = toml_string("Cargo.toml", "description");
    let pyproject = toml_string("pyproject.toml", "description");
    let npm = json_string("npm/package.json.in", "description");

    assert_eq!(
        cargo, pyproject,
        "Cargo.toml and pyproject.toml disagree; pyproject.toml is the one PyPI shows"
    );
    assert_eq!(
        cargo, npm,
        "Cargo.toml and npm/package.json.in disagree; the latter is the one npm shows"
    );
}

#[test]
fn the_description_says_what_the_crate_is_called() {
    // Cheap guard against a description edited into something that no longer
    // describes this package at all.
    let d = toml_string("Cargo.toml", "description");
    assert!(d.len() > 40, "description is too short to be one: {d:?}");
    assert!(!d.ends_with(' '), "description has a trailing space");
    assert!(d.contains("Orthodox"), "description no longer mentions Orthodox: {d:?}");
}

/// The opening paragraph of a README, unwrapped to one line.
fn lede(path: &str) -> String {
    let text = fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let body = text
        .split("\n\n")
        .find(|p| !p.trim().is_empty() && !p.trim_start().starts_with('#'))
        .unwrap_or_else(|| panic!("{path}: no opening paragraph"));
    body.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn both_readmes_open_with_the_description() {
    // A registry page leads with the README, not with the description field,
    // so the sentence a reader actually meets is this one. npm's README is a
    // separate file from the crate's and had gone its own way: it described
    // the package without mentioning that it covers Byzantine Catholic
    // churches at all.
    let description = toml_string("Cargo.toml", "description");
    assert_eq!(
        lede("README.md"),
        description,
        "README.md does not open with the published description"
    );
    assert_eq!(
        lede("npm/README.md"),
        description,
        "npm/README.md does not open with the published description"
    );
}
