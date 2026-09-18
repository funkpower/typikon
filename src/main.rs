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

    /// YYYY-MM-DD (default: today)
    #[arg(short, long)]
    date: Option<String>,

    /// How many days from the start date, inclusive
    #[arg(short = 'n', long, default_value_t = 1)]
    days: i32,

    /// How to print it
    #[arg(short = 'f', long, value_enum, default_value_t = Format::Text)]
    format: Format,

    /// Shorthand for --format json
    #[arg(long, conflicts_with = "ics")]
    json: bool,

    /// Shorthand for --format ics
    #[arg(long)]
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
    #[arg(long)]
    indent: Option<usize>,

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
}

fn elide(text: &str, width: usize) -> String {
    if text.chars().count() <= width {
        return text.to_string();
    }
    let cut: String = text.chars().take(width - 1).collect();
    let head = match cut[cut.len() / 2..].find(' ') {
        Some(_) => match cut.trim_end().rfind(' ') {
            Some(i) => &cut[..i],
            None => &cut,
        },
        None => &cut,
    };
    format!("{}\u{2026}", head.trim_end_matches([' ', ',', ';']))
}

fn print_days(days: &[Day], jur: &Jurisdiction, source: Source, verbose: bool) {
    let rule = rules::get(jur.rule);
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
        println!("  {} \u{b7} {} \u{b7} {} usage{from}", jur.name, rule.title, jur.tradition.key());
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

    println!("\n  {} \u{b7} {}\n", jur.name, rule.title);
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
        let where_ = config::write(jur.key).map_err(|e| e.to_string())?;
        println!("  {} \u{2014} written to {}", jur.name, where_.display());
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
    let jur: &'static Jurisdiction = match &args.tradition {
        None => base,
        Some(t) => {
            let tradition = jurisdictions::Tradition::parse(t)
                .ok_or_else(|| format!("unknown tradition {t:?}; try slavic or greek"))?;
            Box::leak(Box::new(Jurisdiction { tradition, ..*base }))
        }
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
            println!("{}", to_json(&days, args.indent));
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
        if let Ok(p) = config::write(jur.key) {
            println!("  Reckoning by the {}, written to {}.", jur.name.to_lowercase(), p.display());
            println!("  Change it with `typikon --set-default <jurisdiction>`, or see `typikon --list`.\n");
        }
    }
    Ok(0)
}

/// Today, from the system clock, without pulling in a date library for it.
fn today() -> Date {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    // Local midnight, via the offset the system reports for now.
    let local = secs + local_offset_secs(secs);
    Date(2440588 + (local.div_euclid(86400)) as i32)
}

/// Seconds east of UTC, read from /etc/localtime through the `date` command
/// only if TZ arithmetic is needed. Falls back to UTC.
fn local_offset_secs(_now: i64) -> i64 {
    if let Ok(out) = std::process::Command::new("date").arg("+%z").output() {
        let s = String::from_utf8_lossy(&out.stdout);
        let s = s.trim();
        if s.len() == 5 {
            if let (Ok(h), Ok(m)) = (s[1..3].parse::<i64>(), s[3..5].parse::<i64>()) {
                let mag = h * 3600 + m * 60;
                return if s.starts_with('-') { -mag } else { mag };
            }
        }
    }
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
