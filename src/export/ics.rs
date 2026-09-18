//! iCalendar, so the calendar can live where the rest of your year does.
//!
//! Each day becomes one all-day event. The summary is the feast or the
//! principal commemoration; the description carries the fast, the season and
//! the rest of the day's commemorations. Days with nothing to say, no feast and
//! no fast, are left out, because a subscribed calendar that marks all 365 days
//! is one nobody keeps.

use crate::day::Day;
use crate::fasting::{Level, Obligation};

/// A count of octets, which is what RFC 5545 measures content lines in.
///
/// A newtype because the Python this was ported from folded on *characters*
/// instead, and an emoji is four octets: lines went eight over a limit the code
/// believed it was respecting. Counting the wrong unit is not a mistake a name
/// can prevent, but it is one a type can.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Octets(usize);

/// RFC 5545: a content line SHOULD NOT be longer than 75 octets, excluding the
/// line break.
const LIMIT: Octets = Octets(75);

fn octets_of(c: char) -> Octets {
    Octets(c.len_utf8())
}

fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\\' => out.push_str(r"\\"),
            ';' => out.push_str(r"\;"),
            ',' => out.push_str(r"\,"),
            '\n' => out.push_str(r"\n"),
            _ => out.push(c),
        }
    }
    out
}

/// Break a content line on octets, continued with a leading space. The split
/// never falls inside a character: the budget is spent per codepoint, so a
/// four-octet emoji moves to the next line whole.
fn fold(line: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut used = Octets(0);
    for c in line.chars() {
        let n = octets_of(c);
        if Octets(used.0 + n.0) > LIMIT {
            out.push(std::mem::take(&mut cur));
            cur.push(' ');
            used = Octets(1);
        }
        cur.push(c);
        used = Octets(used.0 + n.0);
    }
    out.push(cur);
    out.join("\r\n")
}

pub fn to_ics(days: &[Day], name: &str, stamp: &str) -> String {
    let mut lines: Vec<String> = vec![
        "BEGIN:VCALENDAR".into(),
        "VERSION:2.0".into(),
        "PRODID:-//typikon//EN".into(),
        "CALSCALE:GREGORIAN".into(),
        "METHOD:PUBLISH".into(),
        format!("X-WR-CALNAME:{}", escape(name)),
    ];

    for d in days {
        if d.feast.is_none() && d.fast == Level::None {
            continue;
        }
        let start = d.date.iso().replace('-', "");
        let end = (d.date + 1).iso().replace('-', "");
        let summary = if d.emoji.is_empty() {
            d.title().to_string()
        } else {
            format!("{} {}", d.emoji, d.title())
        };

        let mut body: Vec<String> = Vec::new();
        if d.fast != Level::None {
            body.push(if d.reason.is_empty() {
                format!("Fast: {}", d.fast_label)
            } else {
                format!("Fast: {} ({})", d.fast_label, d.reason)
            });
        }
        if d.obligation == Some(Obligation::Obligatory) {
            body.push("Obligatory.".into());
        }
        if let Some(s) = d.season {
            body.push(format!("Season: {}", s.name()));
        }
        if let Some(c) = &d.cycle {
            let mut chars = c.chars();
            let capped = match chars.next() {
                Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            };
            body.push(capped);
        }
        body.push(format!("Church date: {}", d.church_date));
        for c in &d.commemorations {
            body.push(c.clone());
        }

        lines.extend([
            "BEGIN:VEVENT".into(),
            // The tradition and the discipline change what the event says, so
            // they belong in its identity. Without them a client subscribed to
            // two feeds treats them as one event and keeps whichever arrived
            // last.
            format!(
                "UID:{}-{}-{}-{}@typikon",
                start,
                d.jurisdiction,
                d.tradition,
                if d.shown_at_typikon { "typikon" } else { "particular" }
            ),
            format!("DTSTAMP:{stamp}"),
            format!("DTSTART;VALUE=DATE:{start}"),
            format!("DTEND;VALUE=DATE:{end}"),
            fold(&format!("SUMMARY:{}", escape(&summary))),
            fold(&format!("DESCRIPTION:{}", escape(&body.join("\n")))),
            "TRANSP:TRANSPARENT".into(),
            "END:VEVENT".into(),
        ]);
    }

    lines.push("END:VCALENDAR".into());
    let mut s = lines.join("\r\n");
    s.push_str("\r\n");
    s
}
