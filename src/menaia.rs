//! Supplementary commemorations, fetched rather than shipped.
//!
//! The menologion compiled into this binary is the common Byzantine one,
//! composed for this project and licensed with the rest of it. Some churches
//! also keep their own saints, and those lists are usually published by a body
//! that holds the copyright in the wording. Redistributing them would be taking
//! someone else's work; fetching them onto your own machine, for your own use,
//! is not.
//!
//! So supplementary sets are **not** in the binary. `typikon --update-menaion
//! ruthenian` fetches one into your data directory, where the menologion loader
//! picks it up automatically and overlays it on the common set.
//!
//! The fetch shells out to `curl`, deliberately. Linking an HTTP stack and a
//! TLS stack into a calendar would be several megabytes and a standing audit
//! surface, for one command most people run once.

use std::process::Command;

pub struct Source {
    pub key: &'static str,
    pub title: &'static str,
    pub url: &'static str,
    pub note: &'static str,
}

pub const SOURCES: &[Source] = &[Source {
    key: "ruthenian",
    title: "Byzantine Catholic Metropolia of Pittsburgh",
    url: "https://mci.archpitt.org/liturgy/Calendar_of_Saints.html",
    note: "Published by the Metropolitan Cantor Institute. Fetched for your own \
           use; not redistributed with this program.",
}];

const MONTHS: [&str; 12] = [
    "January", "February", "March", "April", "May", "June", "July", "August",
    "September", "October", "November", "December",
];

pub fn find(name: &str) -> Option<&'static Source> {
    SOURCES.iter().find(|s| s.key == name)
}

fn fetch(url: &str) -> Result<String, String> {
    let out = Command::new("curl")
        .args(["-sSL", "--max-time", "30", "-A", "typikon", url])
        .output()
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => {
                "curl is not on PATH, and the fetch needs it".to_string()
            }
            _ => e.to_string(),
        })?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn strip_html(html: &str) -> Vec<String> {
    let mut text = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut tag = String::new();
    for c in html.chars() {
        match c {
            '<' => {
                in_tag = true;
                tag.clear();
            }
            '>' => {
                in_tag = false;
                let t = tag.trim_start_matches('/').to_ascii_lowercase();
                let t = t.split_whitespace().next().unwrap_or("");
                if matches!(t, "br" | "p" | "div" | "li" | "h1" | "h2" | "h3" | "h4") {
                    text.push('\n');
                }
                text.push('\n');
            }
            _ if in_tag => tag.push(c),
            _ => text.push(c),
        }
    }
    for (from, to) in [
        ("&nbsp;", " "), ("&amp;", "&"), ("&#8217;", "'"),
        ("&rsquo;", "'"), ("&mdash;", "\u{2014}"), ("&quot;", "\""),
    ] {
        text = text.replace(from, to);
    }
    text.lines()
        .map(|l| l.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|l| !l.is_empty())
        .collect()
}

fn month_index(word: &str) -> Option<u32> {
    MONTHS.iter().position(|m| *m == word).map(|i| i as u32 + 1)
}

/// Split a line into (month, day, rest) when it opens a dated entry.
fn dated(line: &str) -> Option<(u32, u32, String)> {
    let mut it = line.splitn(3, ' ');
    let month = month_index(it.next()?)?;
    let day: u32 = it.next()?.trim_end_matches(['.', ',']).parse().ok()?;
    Some((month, day, it.next().unwrap_or("").trim().to_string()))
}

fn is_heading(line: &str) -> bool {
    month_index(line.trim_end_matches('.')).is_some()
}

pub struct Fetched {
    pub path: std::path::PathBuf,
    pub days: usize,
    pub corrections: Vec<String>,
    pub missing: Vec<String>,
}

/// MCI publishes prose, not a table, so entries split on the date headings.
///
/// The page has at least one typo: 14 October is written "October 1", which
/// silently overwrites the real entry for that day and takes the Protection of
/// the Theotokos with it. A date that goes backwards inside a month is the
/// giveaway, and is corrected to follow the previous one.
pub fn update(name: &str) -> Result<Fetched, String> {
    let source = find(name).ok_or_else(|| {
        let known: Vec<_> = SOURCES.iter().map(|s| s.key).collect();
        format!("unknown menaion: {name:?}; try one of {}", known.join(", "))
    })?;

    let lines = strip_html(&fetch(source.url)?);

    // MCI writes the festal cycle inline, as a commemoration among the saints.
    // It repeats for days at a time and is not a saint, so it becomes the day's
    // cycle and leaves the list alone.
    let festal = [
        ("pre-festive", "forefeast"), ("fore-feast", "forefeast"),
        ("post-festive", "afterfeast"), ("after-feast", "afterfeast"),
        ("leave-taking", "leave-taking"),
    ];

    let mut out: std::collections::BTreeMap<String, serde_json::Value> = Default::default();
    let mut corrections = Vec::new();
    let mut cur: Option<(u32, u32)> = None;
    let mut buf: Vec<String> = Vec::new();
    let mut prev: Option<(u32, u32)> = None;

    let flush = |cur: &Option<(u32, u32)>, buf: &mut Vec<String>,
                     out: &mut std::collections::BTreeMap<String, serde_json::Value>| {
        let Some((m, d)) = *cur else { buf.clear(); return };
        let mut items: Vec<String> = Vec::new();
        let mut cycle: Option<String> = None;
        for piece in buf.iter() {
            for part in split_sentences(piece) {
                let part = part.trim_matches([' ', '.', ';']).to_string();
                if part.len() <= 3 || dated(&part).is_some() || is_heading(&part) {
                    continue;
                }
                let lower = part.to_ascii_lowercase();
                if let Some((_, word)) = festal.iter().find(|(p, _)| lower.starts_with(p)) {
                    if cycle.is_none() {
                        let rest = part
                            .split_once(' ')
                            .map(|(_, tail)| tail)
                            .unwrap_or("")
                            .trim_start_matches("Day ")
                            .trim_start_matches("of ")
                            .trim_matches([' ', '.', ','])
                            .to_string();
                        cycle = Some(if rest.is_empty() {
                            word.to_string()
                        } else {
                            format!("{word} of {rest}")
                        });
                    }
                    continue;
                }
                items.push(part);
            }
        }
        buf.clear();
        if items.is_empty() && cycle.is_none() {
            return;
        }
        let mut entry = serde_json::Map::new();
        if !items.is_empty() {
            entry.insert("commemorations".into(), serde_json::json!(items));
        }
        if let Some(c) = cycle {
            entry.insert("cycle".into(), serde_json::json!(c));
        }
        out.insert(format!("{m:02}-{d:02}"), serde_json::Value::Object(entry));
    };

    for line in &lines {
        if is_heading(line) {
            continue;
        }
        if let Some((month, mut day, rest)) = dated(line) {
            flush(&cur, &mut buf, &mut out);
            if let Some((pm, pd)) = prev {
                if pm == month && day <= pd {
                    corrections.push(format!("{month:02}-{day:02} -> {month:02}-{:02}", pd + 1));
                    day = pd + 1;
                }
            }
            prev = Some((month, day));
            cur = Some((month, day));
            if !rest.is_empty() {
                buf.push(rest);
            }
        } else if cur.is_some() {
            buf.push(line.clone());
        }
    }
    flush(&cur, &mut buf, &mut out);

    let mut missing = Vec::new();
    for m in 1..=12u32 {
        for d in 1..=crate::calendars::days_in_month(2024, m) {
            let key = format!("{m:02}-{d:02}");
            if !out.contains_key(&key) {
                missing.push(key);
            }
        }
    }

    // This is a complete calendar published by that church, so it stands in for
    // the common commemorations rather than piling on top of them.
    let mut payload = serde_json::Map::new();
    payload.insert(
        "_meta".into(),
        serde_json::json!({"mode": "replace", "source": source.url, "title": source.title}),
    );
    for (k, v) in &out {
        payload.insert(k.clone(), v.clone());
    }

    let dir = crate::menologion::data_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{name}.json"));
    let text = serde_json::to_string_pretty(&serde_json::Value::Object(payload))
        .map_err(|e| e.to_string())?;
    std::fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))?;

    Ok(Fetched { path, days: out.len(), corrections, missing })
}

/// Split on sentence ends that begin a new capitalised clause, the way the MCI
/// page separates commemorations inside one paragraph.
fn split_sentences(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut parts = Vec::new();
    let mut start = 0usize;
    let mut i = 0usize;
    while i + 2 < chars.len() {
        if chars[i] == '.' && chars[i + 1] == ' ' {
            let mut j = i + 1;
            while j < chars.len() && chars[j] == ' ' {
                j += 1;
            }
            if j < chars.len() && (chars[j].is_ascii_uppercase() || chars[j] == '(') {
                parts.push(chars[start..=i].iter().collect());
                start = j;
                i = j;
                continue;
            }
        }
        i += 1;
    }
    parts.push(chars[start..].iter().collect());
    parts
}
