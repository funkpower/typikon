//! The command line.

use clap::{Parser, ValueEnum};

use typikon::calendars::Date;
use typikon::config::{self, Source};
use typikon::day::{week, year_of, Day};
use typikon::export::{to_delimited, to_ics, to_json};
use typikon::jurisdictions::{self, Jurisdiction, CALENDARS, JURISDICTIONS};
use typikon::menaia;
use typikon::rules;

/// The shapes this can print in. `text` is for a person, `json` for a front
/// end, `csv` and `tsv` for a spreadsheet or a pipeline, `ics` for a calendar
/// you subscribe to.
#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Format {
    Text,
    Json,
    Csv,
    Tsv,
    Ics,
}

#[derive(Parser)]
#[command(
    name = "typikon",
    version,
    about = "The Orthodox liturgical day: feasts, seasons and the fast."
)]
struct Args {
    /// A church or a reckoning; see --list
    #[arg(short = 'j', long, visible_alias = "calendar", short_alias = 'c')]
    jurisdiction: Option<String>,

    /// Greek or Slavic usage, overriding the jurisdiction's own
    #[arg(short = 't', long)]
    tradition: Option<String>,

    /// Show the day at the Typikon's full discipline rather than only what
    /// your church's particular law requires. Obligations are still reported.
    #[arg(long, conflicts_with = "no_typikon")]
    typikon: bool,

    /// Show only what your church's particular law requires, overriding a
    /// `typikon = true` remembered in the config.
    #[arg(long)]
    no_typikon: bool,

    /// YYYY-MM-DD (default: today)
    #[arg(short, long)]
    date: Option<String>,

    /// How many days from the start date, inclusive
    #[arg(short = 'n', long, default_value_t = 1, value_parser = clap::value_parser!(i32).range(1..=36600))]
    days: i32,

    /// How to print it
    #[arg(short = 'f', long, value_enum, default_value_t = Format::Text)]
    format: Format,

    /// Shorthand for --format json
    #[arg(long, conflicts_with_all = ["ics", "format"])]
    json: bool,

    /// Shorthand for --format ics
    #[arg(long, conflicts_with = "format")]
    ics: bool,

    /// Leave the glyphs out, for a terminal or a bar without them
    #[arg(long)]
    no_emoji: bool,

    /// Columns for csv and tsv, comma-separated; see --list-fields
    #[arg(long, value_name = "LIST", value_delimiter = ',')]
    fields: Vec<String>,

    /// List the columns csv and tsv can print
    #[arg(long)]
    list_fields: bool,

    /// Indent JSON output
    #[arg(long, value_parser = clap::value_parser!(u8).range(0..=16))]
    indent: Option<u8>,

    /// Print a year's key dates and exit
    #[arg(long)]
    year: Option<i32>,

    /// List every jurisdiction and calendar
    #[arg(long)]
    list: bool,

    /// Remember a jurisdiction and exit
    #[arg(long, value_name = "JURISDICTION")]
    set_default: Option<String>,

    /// Fetch a supplementary commemoration set
    #[arg(long, value_name = "NAME")]
    update_menaion: Option<String>,

    /// List the supplementary sets that can be fetched
    #[arg(long)]
    list_menaia: bool,

    #[arg(short, long)]
    verbose: bool,
}

fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        if !line.is_empty() && line.chars().count() + word.chars().count() + 1 > width {
            out.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        out.push(line);
    }
    out
}

fn list_jurisdictions() {
    println!("\n  Calendars, a reckoning without naming a church:\n");
    for j in CALENDARS {
        println!("    {:20} {}", j.key, j.name);
        for line in wrap(j.note, 56) {
            println!("    {:20} {}", "", line);
        }
    }
    println!("\n  Jurisdictions:\n");
    let width = JURISDICTIONS.iter().map(|j| j.key.len()).max().unwrap_or(12);
    for j in JURISDICTIONS {
        let cal = match j.calendar() {
            jurisdictions::Calendar::Old => "old calendar",
            jurisdictions::Calendar::New => "new calendar",
            jurisdictions::Calendar::Western => "gregorian",
        };
        let rule = rules::get(j.rule).name;
        println!("    {:width$}  {}", j.key, j.name);
        println!("    {:width$}  {cal}; {rule} fasting rule", "");
        for line in wrap(j.note, 66) {
            println!("    {:width$}  {line}", "");
        }
        println!();
    }
    println!("  A church not listed here keeps one of the three reckonings above.");
    println!("  Which one follows from two questions: is Pascha kept with Rome or");
    println!("  with the local Orthodox, and are the fixed feasts on the civil");
    println!("  calendar or thirteen days behind it?\n");
}

/// Cut a title to `width` characters, on a word boundary where there is one,
/// and mark it with an ellipsis.
///
/// The halfway point has to be found with `char_indices`. Taking it as
/// `cut.len() / 2` is a byte index into a string counted in characters, and
/// every movable feast title carries an em dash, so at the widths where the
/// midpoint landed inside one this panicked: 80 columns among them.
pub fn elide(text: &str, width: usize) -> String {
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
    // Only break on a word if one falls in the second half; otherwise a single
    // long word would be cut back to almost nothing.
    let head = match cut[midpoint..].find(' ') {
        Some(_) => match cut.trim_end().rfind(' ') {
            Some(i) => &cut[..i],
            None => &cut,
        },
        None => &cut,
    };
    format!("{}\u{2026}", head.trim_end_matches([' ', ',', ';']))
}

/// How this run's reckoning reads, for the header of either view.
fn describe_reckoning(jur: &Jurisdiction) -> String {
    let rule = rules::get(jur.rule);
    match jur.discipline.filter(|d| *d != jur.rule) {
        None => rule.title.to_string(),
        Some(d) => format!("{}, obliging under {}", rules::get(d).title, rule.title),
    }
}

fn print_days(days: &[Day], jur: &Jurisdiction, source: Source, verbose: bool) {
    if let [d] = days {
        let mut head = format!("  {} {}", d.weekday.name(), d.date.long());
        let civil_md = format!("{} {}", d.date.day(), typikon::calendars::MONTHS[(d.date.month() - 1) as usize]);
        if d.church_date != civil_md {
            head.push_str(&format!("   \u{2014} {} in the church calendar", d.church_date));
        }
        println!("\n{head}");
        let from = match source {
            Source::Flag => String::new(),
            Source::Environment => "   (from the environment)".into(),
            Source::Config => "   (from config)".into(),
            Source::Default => String::new(),
        };
        println!(
            "  {} \u{b7} {} \u{b7} {} usage{from}",
            jur.name,
            describe_reckoning(jur),
            jur.tradition.key()
        );
        println!();
        let sign = d.rank.map(|r| format!("{} ", r.sign())).unwrap_or_default();
        if d.emoji.is_empty() {
            println!("  {sign}{}", d.title());
        } else {
            println!("  {}  {sign}{}", d.emoji, d.title());
        }
        if let Some(c) = &d.cycle {
            println!("      {c}");
        }
        println!();
        let mark = if d.is_obligatory() { "  \u{2014} obligatory" } else { "" };
        println!("      {}{mark}", d.fast_label);
        if !d.reason.is_empty() {
            println!("      {}", d.reason);
        }
        if let Some(s) = d.season {
            println!("      {}", s.name());
        }
        let rest: Vec<_> = d.commemorations.iter().filter(|c| c.as_str() != d.title()).collect();
        if !rest.is_empty() {
            println!();
            for c in rest {
                println!("      \u{b7} {c}");
            }
        }
        if verbose {
            println!();
            println!("      Pascha {}  \u{b7}  {} days", d.pascha, d.days_to_pascha);
        }
        println!();
        return;
    }

    println!("\n  {} \u{b7} {}\n", jur.name, describe_reckoning(jur));
    // The glyph is two columns wide in a terminal and one char to Rust, which
    // is why the arithmetic below looks off by one.
    let fixed = 2 + 2 + 10 + 1 + 1 + 32;
    let cols = std::env::var("COLUMNS").ok().and_then(|c| c.parse().ok()).unwrap_or(100usize);
    let room = cols.saturating_sub(fixed + 1).max(24);
    for d in days {
        let mark = if d.is_obligatory() { "*" } else { " " };
        // The glyph column collapses entirely when there is no glyph, rather
        // than leaving a hole where one used to be.
        let glyph = if d.emoji.is_empty() { String::new() } else { format!("{} ", d.emoji) };
        println!(
            "  {glyph}{:>10}{mark} {:<32}{}",
            d.date.short(),
            d.fast_label,
            elide(d.title(), room)
        );
    }
    if days.iter().any(Day::is_obligatory) {
        println!("\n  * obligatory. Everything unmarked is customary and does not bind.");
    }
    println!();
}

fn run() -> Result<i32, String> {
    let args = Args::parse();

    if args.list {
        list_jurisdictions();
        return Ok(0);
    }

    if args.list_fields {
        println!("\n  Columns for --format csv and --format tsv:\n");
        for c in typikon::export::csv::COLUMNS {
            println!("    {c}");
        }
        println!("\n  Pick a subset with --fields date,fast,title\n");
        return Ok(0);
    }

    if args.list_menaia {
        println!("\n  Supplementary commemorations, fetched on request:\n");
        for s in menaia::SOURCES {
            println!("    {:14} {}", s.key, s.title);
            println!("    {:14} {}", "", s.url);
            for line in wrap(s.note, 62) {
                println!("    {:14} {line}", "");
            }
            println!();
        }
        println!("  They are written to {}\n", typikon::menologion::data_dir().display());
        return Ok(0);
    }

    if let Some(name) = &args.update_menaion {
        let f = menaia::update(name)?;
        println!("  {} days written to {}", f.days, f.path.display());
        if let Some(s) = menaia::find(name) {
            println!("  Source: {}", s.title);
        }
        if !f.corrections.is_empty() {
            println!("  source typos corrected: {}", f.corrections.join(", "));
        }
        if !f.missing.is_empty() {
            println!("  MISSING: {}", f.missing.join(", "));
            return Ok(1);
        }
        return Ok(0);
    }

    if let Some(name) = &args.set_default {
        let jur = jurisdictions::resolve(Some(name))
            .map_err(|n| format!("unknown jurisdiction {n:?}; try --list"))?;
        // Whatever else was given alongside is remembered too, so that
        // `--set-default ruthenian --typikon` records the whole setup rather
        // than half of it and leaves a terminal disagreeing with a widget.
        let tradition = match &args.tradition {
            None => None,
            Some(t) => Some(
                jurisdictions::Tradition::parse(t)
                    .ok_or_else(|| format!("unknown tradition {t:?}; try slavic or greek"))?
                    .key()
                    .to_string(),
            ),
        };
        let saved = config::Saved {
            jurisdiction: Some(jur.key.to_string()),
            tradition,
            typikon: args.typikon,
        };
        let where_ = config::write(&saved).map_err(|e| e.to_string())?;
        println!("  {} \u{2014} written to {}", jur.name, where_.display());
        if let Some(t) = &saved.tradition {
            println!("  {t} usage");
        }
        if saved.typikon {
            println!("  shown at the Typikon's discipline");
        }
        return Ok(0);
    }

    let (name, source) = match &args.jurisdiction {
        Some(j) => (Some(j.clone()), Source::Flag),
        None => config::default_jurisdiction(),
    };
    let base = jurisdictions::resolve(name.as_deref())
        .map_err(|n| format!("unknown jurisdiction {n:?}; try --list"))?;

    // A tradition given on the command line overrides the one the jurisdiction
    // carries. Leaked as a 'static because a Jurisdiction is otherwise a
    // compile-time constant, and one override per run is not worth threading
    // a lifetime through the whole library for.
    let saved = config::saved();
    let tradition = match args.tradition.as_deref().or(saved.tradition.as_deref()) {
        None => base.tradition,
        Some(t) => jurisdictions::Tradition::parse(t)
            .ok_or_else(|| format!("unknown tradition {t:?}; try slavic or greek"))?,
    };
    // A flag either way beats the file; with neither, the file decides.
    let typikon = if args.no_typikon {
        false
    } else {
        args.typikon || saved.typikon
    };
    let discipline = if typikon {
        Some(jurisdictions::RuleKind::Orthodox)
    } else {
        base.discipline
    };
    let jur: &'static Jurisdiction =
        if tradition == base.tradition && discipline == base.discipline {
            base
        } else {
            Box::leak(Box::new(Jurisdiction { tradition, discipline, ..*base }))
        };

    if let Some(year) = args.year {
        if !(1..=9999).contains(&year) {
            return Err(format!("{year} is not a year this calendar reaches"));
        }
        let (mv, fixed) = year_of(year, jur);
        println!("\n  {year} \u{b7} {}\n", jur.name);
        for (label, d) in [
            ("publican pharisee", mv.publican_pharisee),
            ("meatfare", mv.meatfare),
            ("cheesefare", mv.cheesefare),
            ("clean monday", mv.clean_monday),
            ("great fast end", mv.great_fast_end),
            ("lazarus saturday", mv.lazarus_saturday),
            ("palm sunday", mv.palm_sunday),
            ("great friday", mv.great_friday),
            ("pascha", mv.pascha),
            ("ascension", mv.ascension),
            ("pentecost", mv.pentecost),
            ("all saints", mv.all_saints),
            ("apostles start", mv.apostles_start),
        ] {
            println!("    {label:20} {d}  {}", d.weekday().short());
        }
        for (label, d) in fixed {
            println!("    {label:20} {d}  {}", d.weekday().short());
        }
        println!();
        return Ok(0);
    }

    let start = match &args.date {
        Some(s) => Date::parse_iso(s)
            .ok_or_else(|| format!("{s:?} is not a date \u{2014} expected YYYY-MM-DD"))?,
        None => today(),
    };
    let mut days = week(start, args.days, jur);
    if args.no_emoji {
        for d in &mut days {
            d.emoji = "";
        }
    }

    // The boolean flags predate --format and still mean what they did.
    let format = if args.json {
        Format::Json
    } else if args.ics {
        Format::Ics
    } else {
        args.format
    };

    match format {
        Format::Json => {
            println!("{}", to_json(&days, args.indent.map(usize::from)));
            return Ok(0);
        }
        Format::Ics => {
            print!("{}", to_ics(&days, &format!("{} \u{2014} Typikon", jur.name), &stamp()));
            return Ok(0);
        }
        Format::Csv | Format::Tsv => {
            if let Some(bad) = typikon::export::csv::unknown_column(&args.fields) {
                return Err(format!("no such column {bad:?}; try --list-fields"));
            }
            let sep = if format == Format::Csv { ',' } else { '\t' };
            print!("{}", to_delimited(&days, sep, &args.fields));
            return Ok(0);
        }
        Format::Text => {}
    }

    print_days(&days, jur, source, args.verbose);

    // The first run should say what it assumed, and then stop saying it.
    if source == Source::Default {
        let first = config::Saved {
            jurisdiction: Some(jur.key.to_string()),
            ..Default::default()
        };
        if let Ok(p) = config::write(&first) {
            println!("  Reckoning by the {}, written to {}.", jur.name.to_lowercase(), p.display());
            println!("  Change it with `typikon --set-default <jurisdiction>`, or see `typikon --list`.\n");
        }
    }
    Ok(0)
}

/// Today, from the system clock, in local time.
///
/// The local offset comes from `date +%z`, because working it out from
/// /etc/localtime means parsing TZif and this program would rather shell out
/// than carry a time zone database. Where `date` is absent, as in a scratch
/// container, the answer is UTC and it says so on stderr rather than quietly
/// reporting the wrong day either side of midnight.
fn today() -> Date {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let local = secs + local_offset_secs();
    // Unix epoch is JDN 2440588. The clock cannot reach either bound.
    Date::from_jdn(2440588 + local.div_euclid(86400) as i32).unwrap_or(Date::MIN)
}

/// Seconds east of UTC, or None when it could not be determined.
fn local_offset_secs() -> i64 {
    let Ok(out) = std::process::Command::new("date").arg("+%z").output() else {
        eprintln!("typikon: `date` is not on PATH; reckoning today in UTC");
        return 0;
    };
    let text = String::from_utf8_lossy(&out.stdout);
    let s = text.trim();
    // "+HHMM". Checked as bytes so a short or multi-byte reply cannot panic a
    // slice, which a str index here would.
    let b = s.as_bytes();
    if b.len() == 5 && (b[0] == b'+' || b[0] == b'-') && b[1..].iter().all(u8::is_ascii_digit) {
        let h = (s[1..3].parse::<i64>()).unwrap_or(0);
        let m = (s[3..5].parse::<i64>()).unwrap_or(0);
        let mag = h * 3600 + m * 60;
        return if b[0] == b'-' { -mag } else { mag };
    }
    eprintln!("typikon: could not read the local time offset; reckoning today in UTC");
    0
}

fn stamp() -> String {
    let out = std::process::Command::new("date")
        .args(["-u", "+%Y%m%dT%H%M%SZ"])
        .output();
    match out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => "19700101T000000Z".into(),
    }
}

fn main() {
    match run() {
        Ok(code) => std::process::exit(code),
        Err(msg) => {
            eprintln!("typikon: {msg}");
            std::process::exit(2);
        }
    }
}
