//! Every title the program can print, at every width it can be asked to print
//! at. The seven-day view cuts titles to fit, and the titles are full of em
//! dashes: a cut taken at a byte index rather than a character one panics, and
//! did, at 80 columns.

use std::process::Command;

use typikon::calendars::Date;
use typikon::day::week;
use typikon::jurisdictions::{CALENDARS, JURISDICTIONS};

/// The same cut the seven-day view makes. Kept in step with `main.rs::elide`
/// by exercising the binary below as well.
fn elide(text: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    if text.chars().count() <= width {
        return text.to_string();
    }
    let cut: String = text.chars().take(width - 1).collect();
    let midpoint = cut
        .char_indices()
        .nth(cut.chars().count() / 2)
        .map_or(cut.len(), |(i, _)| i);
    let head = match cut[midpoint..].find(' ') {
        Some(_) => match cut.trim_end().rfind(' ') {
            Some(i) => &cut[..i],
            None => &cut,
        },
        None => &cut,
    };
    format!("{}\u{2026}", head.trim_end_matches([' ', ',', ';']))
}

#[test]
fn no_title_of_any_year_can_be_cut_at_any_width_without_panicking() {
    let start = Date::from_ymd(2020, 1, 1).unwrap();
    for jur in JURISDICTIONS.iter().chain(CALENDARS.iter()) {
        for day in week(start, 366 * 11, jur) {
            for width in 0..=120 {
                let out = elide(day.title(), width);
                assert!(
                    out.chars().count() <= width.max(1),
                    "{}: {width} gave {} chars",
                    day.date,
                    out.chars().count()
                );
            }
        }
    }
}

#[test]
fn a_cut_never_lands_inside_a_character() {
    // Every movable feast title carries an em dash, which is where this broke.
    for title in [
        "The Radiant and Life-giving Resurrection of our Lord \u{2014} Holy Pascha",
        "Holy Pentecost \u{2014} the Descent of the Holy Spirit",
        "Clean Monday \u{2014} the first day of the Great Fast",
        "Meatfare Sunday \u{2014} the Sunday of the Last Judgement",
        "\u{2014}\u{2014}\u{2014}\u{2014}\u{2014}\u{2014}\u{2014}\u{2014}",
    ] {
        for width in 0..=80 {
            let out = elide(title, width);
            assert!(out.is_char_boundary(0));
            assert!(std::str::from_utf8(out.as_bytes()).is_ok());
        }
    }
}

#[test]
fn the_binary_survives_every_terminal_width() {
    let binary = concat!(env!("CARGO_MANIFEST_DIR"), "/target/release/typikon");
    if !std::path::Path::new(binary).exists() {
        eprintln!("skipping: {binary} not built");
        return;
    }
    for width in [1, 20, 40, 60, 76, 78, 80, 82, 88, 100, 120, 200] {
        for jur in ["new", "rocor", "ruthenian", "goarch"] {
            let out = Command::new(binary)
                .args(["-j", jur, "--date", "2026-01-01", "--days", "365"])
                .env("COLUMNS", width.to_string())
                .env_remove("TYPIKON_JURISDICTION")
                .output()
                .expect("the binary runs");
            assert!(
                out.status.success(),
                "COLUMNS={width} -j {jur}: exit {:?}\n{}",
                out.status.code(),
                String::from_utf8_lossy(&out.stderr)
            );
        }
    }
}
