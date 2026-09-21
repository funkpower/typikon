//! iCalendar, so the calendar can live where the rest of your year does.
//!
//! Each day becomes one all-day event. The summary is the feast or the
//! principal commemoration; the description lists every commemoration, then the
//! fast, then where the day sits in the year.
//!
//! By default the days with nothing to say, no feast and no fast, are left out,
//! because a subscribed calendar that marks all 365 days is one nobody keeps.
//! `Ics::every_day` is for the other kind of reader, who wants the saint of
//! each morning and does not mind a full calendar: it gives every day an event,
//! ferial ones included.

use crate::day::Day;
use crate::fasting::{Level, Obligation};

/// How to write the calendar.
pub struct Ics<'a> {
    /// What a client shows the subscription as.
    pub name: &'a str,
    /// `DTSTAMP`, as `YYYYMMDDTHHMMSSZ`.
    pub stamp: &'a str,
    /// One event for every day, rather than only the days carrying a feast or
    /// a fast.
    pub every_day: bool,
}

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

/// The calendar as it has always been written: a subscription, thinned to the
/// days worth marking.
pub fn to_ics(days: &[Day], name: &str, stamp: &str) -> String {
    to_ics_with(days, &Ics { name, stamp, every_day: false })
}

pub fn to_ics_with(days: &[Day], opts: &Ics<'_>) -> String {
    let mut lines: Vec<String> = vec![
        "BEGIN:VCALENDAR".into(),
        "VERSION:2.0".into(),
        "PRODID:-//typikon//EN".into(),
        "CALSCALE:GREGORIAN".into(),
        "METHOD:PUBLISH".into(),
        format!("X-WR-CALNAME:{}", escape(opts.name)),
    ];

    for d in days {
        if !opts.every_day && d.feast.is_none() && d.fast == Level::None {
            continue;
        }
        let start = d.date.iso().replace('-', "");
        let end = (d.date + 1).iso().replace('-', "");
        // The glyph for the fast and the mark for the rank, in the order the
        // text view prints them, so one is scannable down a month view and the
        // other says at a glance whether the day is kept.
        let sign = d.rank.map(|r| r.sign()).unwrap_or("");
        let summary = [d.emoji, sign, d.title()]
            .iter()
            .filter(|p| !p.is_empty())
            .copied()
            .collect::<Vec<_>>()
            .join(" ");

        // Paragraphs, joined by a blank line. A calendar client shows this in
        // a popover a few lines tall, and the saints are what the reader came
        // for, so they go first.
        let mut body: Vec<String> = Vec::new();
        if !d.commemorations.is_empty() {
            body.push(
                d.commemorations
                    .iter()
                    .map(|c| format!("\u{2022} {c}"))
                    .collect::<Vec<_>>()
                    .join("\n"),
            );
        }

        let mut fast = if d.reason.is_empty() {
            format!("Fast: {}", d.fast_label)
        } else {
            format!("Fast: {} ({})", d.fast_label, d.reason)
        };
        if d.obligation == Some(Obligation::Obligatory) {
            fast.push_str("\nObligatory.");
        }
        body.push(fast);

        let mut where_ = Vec::new();
        if let Some(w) = &d.week {
            where_.push(capitalised(w));
        }
        if let Some(s) = d.season {
            where_.push(format!("Season: {}", s.name()));
        }
        if let Some(c) = &d.cycle {
            where_.push(capitalised(c));
        }
        where_.push(format!("Church date: {}", d.church_date));
        body.push(where_.join("\n"));

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
            format!("DTSTAMP:{}", opts.stamp),
            format!("DTSTART;VALUE=DATE:{start}"),
            format!("DTEND;VALUE=DATE:{end}"),
            fold(&format!("SUMMARY:{}", escape(&summary))),
            fold(&format!("DESCRIPTION:{}", escape(&body.join("\n\n")))),
            "TRANSP:TRANSPARENT".into(),
            "END:VEVENT".into(),
        ]);
    }

    lines.push("END:VCALENDAR".into());
    let mut s = lines.join("\r\n");
    s.push_str("\r\n");
    s
}

/// A phrase written to sit inside a sentence, promoted to the start of a line.
fn capitalised(phrase: &str) -> String {
    let mut chars = phrase.chars();
    match chars.next() {
        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}
