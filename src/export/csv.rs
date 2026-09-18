//! Delimited output, for the times a calendar is a table.
//!
//! One row per day, one header row, RFC 4180 quoting. `commemorations` is a
//! list, so it is joined with a semicolon: a cell that is itself a table is
//! worse than a cell you have to split.

use crate::day::Day;

pub const COLUMNS: &[&str] = &[
    "date", "weekday", "jurisdiction", "calendar", "church_date", "season",
    "cycle", "feast", "rank", "sign", "tradition", "title", "fast", "fast_label", "emoji",
    "reason", "obligation", "binds", "pascha", "days_to_pascha",
    "julian_offset", "commemorations",
];

fn quote(field: &str, sep: char) -> String {
    let needs = field.contains(sep)
        || field.contains('"')
        || field.contains('\n')
        || field.contains('\r');
    if !needs {
        return field.to_string();
    }
    format!("\"{}\"", field.replace('"', "\"\""))
}

fn cell(day: &Day, column: &str) -> String {
    match column {
        "date" => day.date.iso(),
        "weekday" => day.weekday.name().to_string(),
        "jurisdiction" => day.jurisdiction.to_string(),
        "calendar" => day.calendar.to_string(),
        "church_date" => day.church_date.clone(),
        "season" => day.season.map(|s| s.name()).unwrap_or("").to_string(),
        "cycle" => day.cycle.clone().unwrap_or_default(),
        "feast" => day.feast.clone().unwrap_or_default(),
        "rank" => day.rank.map(|r| r.key()).unwrap_or("").to_string(),
        "sign" => day.rank.map(|r| r.sign()).unwrap_or("").to_string(),
        "tradition" => day.tradition.to_string(),
        "title" => day.title().to_string(),
        "fast" => day.fast.key().to_string(),
        "fast_label" => day.fast_label.to_string(),
        "emoji" => day.emoji.to_string(),
        "reason" => day.reason.clone(),
        "obligation" => match day.obligation {
            Some(crate::fasting::Obligation::Obligatory) => "obligatory".into(),
            Some(crate::fasting::Obligation::Customary) => "customary".into(),
            None => String::new(),
        },
        "binds" => day.binds.map(|b| b.key()).unwrap_or("").to_string(),
        "pascha" => day.pascha.iso(),
        "days_to_pascha" => day.days_to_pascha.to_string(),
        "julian_offset" => day.julian_offset.to_string(),
        "commemorations" => day.commemorations.join("; "),
        _ => String::new(),
    }
}

/// `columns` empty means every column, in the documented order.
pub fn to_delimited(days: &[Day], sep: char, columns: &[String]) -> String {
    let cols: Vec<&str> = if columns.is_empty() {
        COLUMNS.to_vec()
    } else {
        columns.iter().map(String::as_str).collect()
    };
    let mut out = String::new();
    out.push_str(&cols.iter().map(|c| quote(c, sep)).collect::<Vec<_>>().join(&sep.to_string()));
    out.push('\n');
    for day in days {
        let row: Vec<String> = cols.iter().map(|c| quote(&cell(day, c), sep)).collect();
        out.push_str(&row.join(&sep.to_string()));
        out.push('\n');
    }
    out
}

/// Is every name a real column? Returns the first that is not.
pub fn unknown_column(columns: &[String]) -> Option<&String> {
    columns.iter().find(|c| !COLUMNS.contains(&c.as_str()))
}
