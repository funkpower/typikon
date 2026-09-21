//! Pascha, and the movable cycle that hangs off it.
//!
//! Two reckonings are in use. The **Julian Paschalion** is what nearly every
//! Orthodox Church keeps: the old computus, worked in the Julian calendar and
//! then read off onto a civil date. The **Gregorian Paschalion** is the Western
//! one, kept by the Finnish Orthodox Church and by most Byzantine Catholics.
//!
//! Neither is a variant of the other. They are separate tables that happen to
//! agree in some years, 2025 falls on 20 April by both, and can diverge by as
//! much as five weeks.

use crate::calendars::{Date, JulianDate, Weekday, MAX_YEAR, MIN_YEAR};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Paschalion {
    Julian,
    Gregorian,
}

/// Pascha by the Julian Paschalion, as a civil (Gregorian) date.
///
/// Meeus' Julian algorithm yields a date in the *Julian* calendar, which is
/// then read onto the civil one. The type system enforces that step here: the
/// algorithm builds a `JulianDate`, and only `to_civil` produces a `Date`.
pub fn julian_pascha(year: i32) -> Date {
    let a = year % 4;
    let b = year % 7;
    let c = year % 19;
    let d = (19 * c + 15) % 30;
    let e = (2 * a + 4 * b - d + 34) % 7;
    let month = ((d + e + 114) / 31) as u32;
    let day = (((d + e + 114) % 31) + 1) as u32;
    JulianDate { year, month, day }.to_civil()
}

/// Pascha by the Gregorian Paschalion, the Western computus.
pub fn gregorian_pascha(year: i32) -> Date {
    let a = year % 19;
    let b = year / 100;
    let c = year % 100;
    let d = b / 4;
    let e = b % 4;
    let g = (8 * b + 13) / 25;
    let h = (19 * a + b - d - g + 15) % 30;
    let i = c / 4;
    let k = c % 4;
    let l = (32 + 2 * e + 2 * i - h - k) % 7;
    let m = (a + 11 * h + 19 * l) / 433;
    let month = ((h + l - 7 * m + 90) / 25) as u32;
    let day = ((h + l - 7 * m + 33 * month as i32 + 19) % 32) as u32;
    Date::from_ymd(year, month, day).expect("computus yields a real date")
}

pub fn pascha(year: i32, p: Paschalion) -> Date {
    match p {
        Paschalion::Julian => julian_pascha(year),
        Paschalion::Gregorian => gregorian_pascha(year),
    }
}

/// Every movable date the calendar needs, keyed off Pascha.
///
/// All of these are counted in real days from Pascha, so they are civil dates
/// already and need no calendar conversion: the distance from Pascha to
/// Pentecost is fifty days in every reckoning.
///
/// A struct rather than a map, so a typo in a key is a compile error instead
/// of a silent `None` at the far end of the program.
#[derive(Clone, Copy, Debug)]
pub struct Movables {
    pub pascha: Date,

    // the Triodion, counted backwards
    pub publican_pharisee: Date,
    pub prodigal_son: Date,
    pub meatfare: Date,
    pub cheesefare: Date,
    pub cheesefare_week_start: Date,
    pub clean_monday: Date,
    pub orthodoxy_sunday: Date,
    pub gregory_palamas: Date,
    pub cross_veneration: Date,
    pub john_climacus: Date,
    pub mary_of_egypt: Date,
    /// The Forty Days end here, on the Friday before Lazarus Saturday.
    pub great_fast_end: Date,
    pub lazarus_saturday: Date,
    pub palm_sunday: Date,
    pub great_week_start: Date,
    pub great_thursday: Date,
    pub great_friday: Date,
    pub holy_saturday: Date,

    // the Pentecostarion, counted forwards
    pub bright_week_end: Date,
    pub thomas_sunday: Date,
    pub myrrhbearers: Date,
    pub paralytic: Date,
    pub midfeast: Date,
    pub samaritan_woman: Date,
    pub blind_man: Date,
    pub ascension: Date,
    pub fathers_first_council: Date,
    pub pentecost: Date,
    pub pentecost_week_end: Date,
    pub all_saints: Date,
    /// The Apostles' Fast begins the Monday after All Saints. Its end is a
    /// fixed ecclesiastical date, so the jurisdiction resolves that.
    pub apostles_start: Date,
}

pub fn movable_cycle(year: i32, p: Paschalion) -> Movables {
    let d = pascha(year, p);
    Movables {
        pascha: d,
        publican_pharisee: d - 70,
        prodigal_son: d - 63,
        meatfare: d - 56,
        cheesefare_week_start: d - 55,
        cheesefare: d - 49,
        clean_monday: d - 48,
        orthodoxy_sunday: d - 42,
        gregory_palamas: d - 35,
        cross_veneration: d - 28,
        john_climacus: d - 21,
        mary_of_egypt: d - 14,
        great_fast_end: d - 9,
        lazarus_saturday: d - 8,
        palm_sunday: d - 7,
        great_week_start: d - 6,
        great_thursday: d - 3,
        great_friday: d - 2,
        holy_saturday: d - 1,
        bright_week_end: d + 6,
        thomas_sunday: d + 7,
        myrrhbearers: d + 14,
        paralytic: d + 21,
        midfeast: d + 24,
        samaritan_woman: d + 28,
        blind_man: d + 35,
        ascension: d + 39,
        fathers_first_council: d + 42,
        pentecost: d + 49,
        pentecost_week_end: d + 55,
        all_saints: d + 56,
        apostles_start: d + 57,
    }
}

/// Where a day falls in the count of weeks from Pentecost, which is how the
/// books head the long ordinary stretch of the year: "the 32nd Sunday after
/// Pentecost".
///
/// The count runs from the Monday after Pentecost to the eve of the next
/// Sunday of the Publican and the Pharisee, and it is `None` outside that,
/// through the Triodion and the Pentecostarion, where the Sundays have names
/// of their own instead of a number.
///
/// The governing Pentecost is not always the one in this day's civil year: a
/// day in January is counted from the Pentecost of the year before. That is
/// the one place this differs from `days_to_pascha`, which is always reckoned
/// against its own year's Pascha.
pub fn week_after_pentecost(date: Date, p: Paschalion) -> Option<i32> {
    let year = date.year();
    // A day at either end of the range this calendar answers for would need a
    // Pascha outside it. One week unlabelled beats a panic.
    if !(MIN_YEAR + 1..=MAX_YEAR - 1).contains(&year) {
        return None;
    }
    let this = movable_cycle(year, p);
    let (pentecost, next_publican) = if date > this.pentecost {
        (this.pentecost, movable_cycle(year + 1, p).publican_pharisee)
    } else {
        (movable_cycle(year - 1, p).pentecost, this.publican_pharisee)
    };
    if date <= pentecost || date >= next_publican {
        return None;
    }
    Some((date - pentecost + 6) / 7)
}

/// The same count, worded. Lowercase and article-first, like `cycle`, because
/// both are phrases a front end drops into a sentence of its own.
pub fn week_after_pentecost_label(date: Date, weekday: Weekday, p: Paschalion) -> Option<String> {
    let n = week_after_pentecost(date, p)?;
    let unit = if weekday == Weekday::Sunday { "Sunday" } else { "week" };
    Some(format!("the {} {unit} after Pentecost", ordinal(n)))
}

/// English ordinals, for the one place this calendar has to count out loud.
fn ordinal(n: i32) -> String {
    let suffix = match (n % 100, n % 10) {
        (11..=13, _) => "th",
        (_, 1) => "st",
        (_, 2) => "nd",
        (_, 3) => "rd",
        _ => "th",
    };
    format!("{n}{suffix}")
}
