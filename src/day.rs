//! A single day, fully reckoned.
//!
//! `Day` is what everything else in the crate produces and what every front end
//! consumes. It carries the civil date, the ecclesiastical date, the season,
//! the commemorations and the fast, and it carries the jurisdiction that
//! produced it, so a caller can always ask by whose reckoning.

use serde::Serialize;

use crate::calendars::{Date, Weekday, MONTHS};
use crate::fasting::{Fast, Level, Obligation};
use crate::jurisdictions::Jurisdiction;
use crate::menologion::{self, Menologion, Rank};
use crate::paschalion::{movable_cycle, Movables};
use crate::rules::{self, Context};
use crate::seasons::season_for;

#[derive(Clone, Debug)]
pub struct Day {
    pub date: Date,
    pub weekday: Weekday,
    pub jurisdiction: &'static str,
    pub calendar: &'static str,
    /// The ecclesiastical date, as "25 December".
    pub church_date: String,
    pub season: Option<crate::seasons::Season>,
    pub cycle: Option<String>,
    pub feast: Option<String>,
    pub rank: Option<Rank>,
    pub tradition: &'static str,
    pub commemorations: Vec<String>,
    pub fast: Level,
    pub fast_label: &'static str,
    pub emoji: &'static str,
    pub reason: String,
    pub obligation: Option<Obligation>,
    pub binds: Level,
    pub pascha: Date,
    /// Days from this day to Pascha, **signed**, and to the Pascha of this
    /// day's own civil year. Negative after Pascha has passed: 31 December
    /// reports about -263, not the couple of hundred days until the next one.
    pub days_to_pascha: i32,
    /// How far the Julian reckoning currently lags the Gregorian one: 13 until
    /// 14 March 2100, 14 after it. Reported for every jurisdiction, including
    /// the ones that do not use it.
    pub julian_offset: i32,
}

impl Day {
    pub fn is_obligatory(&self) -> bool {
        self.obligation == Some(Obligation::Obligatory)
    }

    /// What to put on one line: the feast if there is one, else the day's
    /// principal commemoration.
    pub fn title(&self) -> &str {
        match &self.feast {
            Some(f) => f,
            None => self.commemorations.first().map(String::as_str).unwrap_or(""),
        }
    }
}

/// The wire format. Flat and stable, because front ends are the thing most
/// likely to be written by someone else and a schema that shifts underneath
/// them is a bad neighbour. Field names match the shape front ends already read.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DayJson<'a> {
    pub date: String,
    pub weekday: &'a str,
    pub jurisdiction: &'a str,
    pub calendar: &'a str,
    pub church_date: &'a str,
    pub season: Option<&'a str>,
    pub cycle: Option<&'a str>,
    pub feast: Option<&'a str>,
    pub rank: Option<&'a str>,
    /// The mark the books print beside a day of this rank.
    pub sign: Option<&'a str>,
    pub tradition: &'a str,
    pub title: &'a str,
    pub commemorations: &'a [String],
    pub fast: &'a str,
    pub fast_label: &'a str,
    pub emoji: &'a str,
    pub reason: &'a str,
    pub obligation: Option<&'a str>,
    pub binds: &'a str,
    pub pascha: String,
    pub days_to_pascha: i32,
    pub julian_offset: i32,
}

impl Day {
    pub fn to_json(&self) -> DayJson<'_> {
        DayJson {
            date: self.date.iso(),
            weekday: self.weekday.name(),
            jurisdiction: self.jurisdiction,
            calendar: self.calendar,
            church_date: &self.church_date,
            season: self.season.map(|s| s.name()),
            cycle: self.cycle.as_deref(),
            feast: self.feast.as_deref(),
            rank: self.rank.map(|r| r.key()),
            sign: self.rank.map(|r| r.sign()),
            tradition: self.tradition,
            title: self.title(),
            commemorations: &self.commemorations,
            fast: self.fast.key(),
            fast_label: self.fast_label,
            emoji: self.emoji,
            reason: &self.reason,
            obligation: self.obligation.map(|o| match o {
                Obligation::Obligatory => "obligatory",
                Obligation::Customary => "customary",
            }),
            binds: self.binds.key(),
            pascha: self.pascha.iso(),
            days_to_pascha: self.days_to_pascha,
            julian_offset: self.julian_offset,
        }
    }
}

/// Reckon one civil date in one jurisdiction.
pub fn describe(date: Date, jur: &'static Jurisdiction) -> Day {
    let men = menologion::for_jurisdiction(jur.tradition, jur.menaia);
    describe_with(date, jur, &men)
}

/// As `describe`, but with a menologion the caller already built. Reckoning a
/// year one day at a time should not rebuild the overlay 365 times.
pub fn describe_with(date: Date, jur: &'static Jurisdiction, men: &Menologion) -> Day {
    let rule = rules::get(jur.rule);
    let (fy, fm, fd) = jur.fixed_of_civil(date);
    let mv = movable_cycle(date.year(), jur.paschalion);
    let apostles_end = jur.civil_of_fixed(date.year(), 6, 28);
    let season = season_for(date, (fm, fd), &mv, apostles_end);

    let entry = men.get(fm, fd).cloned().unwrap_or_default();
    let mut commemorations = entry.commemorations;
    let mut rank = entry.rank;

    // When a movable feast and a fixed one land together, the higher rank is
    // the day's feast and the other joins the commemorations. Letting the
    // movable one win unconditionally lost the Annunciation whenever it fell in
    // Great Week: the Great Feast of the Incarnation demoted to a bullet, and
    // its rank overwritten with the lesser one the fasting rules then read.
    let fixed_feast = rank
        .filter(|r| r.names_the_day())
        .and_then(|_| commemorations.first().cloned());
    let movable = menologion::movable_feast_for(date, &mv);

    let feast: Option<String> = match (movable, &fixed_feast) {
        (Some((mov_title, mov_rank)), Some(fixed)) if rank > mov_rank => {
            let mov = mov_title.to_string();
            if !commemorations.contains(&mov) {
                commemorations.insert(1.min(commemorations.len()), mov);
            }
            Some(fixed.clone())
        }
        (Some((mov_title, mov_rank)), fixed) => {
            if mov_rank >= rank {
                rank = mov_rank;
            }
            if let Some(f) = fixed {
                if !commemorations.contains(f) {
                    commemorations.insert(1.min(commemorations.len()), f.clone());
                }
            }
            Some(mov_title.to_string())
        }
        (None, fixed) => fixed.clone(),
    };

    let ctx = Context {
        civil: date,
        fixed: (fm, fd),
        fixed_year: fy,
        weekday: date.weekday(),
        mv: &mv,
        season,
        feast: feast
            .clone()
            .or_else(|| commemorations.first().cloned())
            .unwrap_or_default(),
        rank,
        jurisdiction: jur,
        tradition: jur.tradition,
    };

    // The discipline shown and the law that binds are two questions. Normally
    // one tradition answers both. When `discipline` names another, the level
    // and its reason come from that one and the obligation still comes from
    // this church's own law, because that is the part only it can say.
    let shown = jur.discipline.map(rules::get).unwrap_or(rule);
    let mut fast = (shown.fast_for)(&ctx);
    if !std::ptr::eq(shown, rule) {
        fast.obligation = match rule.obligation_for.and_then(|f| f(&ctx)) {
            Some((reason, binds)) => {
                fast.binds = binds;
                fast.reason = format!("{} \u{2014} {reason}", fast.reason);
                Some(Obligation::Obligatory)
            }
            None => {
                fast.binds = Level::None;
                if fast.level == Level::None { None } else { Some(Obligation::Customary) }
            }
        };
    }
    let Fast { level, reason, obligation, binds } = fast;

    Day {
        date,
        weekday: date.weekday(),
        jurisdiction: jur.key,
        calendar: jur.calendar().key(),
        church_date: format!("{} {}", fd, MONTHS[(fm - 1) as usize]),
        season,
        cycle: entry.cycle,
        feast,
        rank,
        tradition: jur.tradition.key(),
        commemorations,
        fast: level,
        fast_label: (shown.vocabulary.label)(level),
        emoji: (shown.vocabulary.emoji)(level),
        reason,
        obligation,
        binds,
        pascha: mv.pascha,
        days_to_pascha: mv.pascha - date,
        julian_offset: date.julian_offset(),
    }
}

/// `days` consecutive days from `start`.
pub fn week(start: Date, days: i32, jur: &'static Jurisdiction) -> Vec<Day> {
    let men = menologion::for_jurisdiction(jur.tradition, jur.menaia);
    (0..days.max(1))
        .map(|n| describe_with(start + n, jur, &men))
        .collect()
}

/// Every key date of a year, for checking against a published calendar.
pub fn year_of(year: i32, jur: &'static Jurisdiction) -> (Movables, [(&'static str, Date); 4]) {
    let mv = movable_cycle(year, jur.paschalion);
    let fixed = [
        ("apostles end", jur.civil_of_fixed(year, 6, 28)),
        ("dormition", jur.civil_of_fixed(year, 8, 15)),
        ("nativity", jur.civil_of_fixed(year, 12, 25)),
        ("theophany", jur.civil_of_fixed(year, 1, 6)),
    ];
    (mv, fixed)
}
