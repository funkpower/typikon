//! Conversion between the civil calendar and the calendar a church reckons in.
//!
//! Every date in this library exists twice. There is the **civil** date, what a
//! phone says, always Gregorian; and the **ecclesiastical** date, the day the
//! Church calls it. For a church on the new calendar those are the same date.
//! For one on the old calendar they differ by the drift between the Julian and
//! Gregorian reckonings: thirteen days from 1900 until 14 March 2100, and
//! fourteen from the 15th, the day after a Julian 29 February the Gregorian
//! calendar does not have.
//!
//! Keeping both is what makes a single menologion serve every jurisdiction. The
//! Nativity is 25 December in the Julian and the Gregorian books alike; the two
//! traditions disagree about which morning that is, not about which date it is.
//!
//! Everything here is built on the Julian Day Number, which is why this crate
//! carries its own date type instead of a calendar library: the conversions the
//! liturgical year needs *are* JDN arithmetic, and a `Date` that is a JDN in a
//! newtype makes "add a week" and "what is this in the Julian calendar" the
//! same two lines.

use std::fmt;

/// A civil (proleptic Gregorian) date, stored as a Julian Day Number.
///
/// A newtype over the JDN rather than a (y, m, d) triple, so that day
/// arithmetic cannot go wrong at a month or year boundary.
/// The years this calendar answers for. The lower bound is the first year the
/// Julian reckoning is meaningful for a Christian calendar; the upper is the
/// one `datetime.date` stops at, so the Python bindings cannot be handed a day
/// they could not return.
pub const MIN_YEAR: i32 = 1;
pub const MAX_YEAR: i32 = 9999;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Date(i32);

/// A date in the Julian calendar. Deliberately a *different type* from `Date`:
/// mixing the two is the single most common way calendar code goes wrong, and
/// the compiler can simply forbid it.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct JulianDate {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

pub const MONTHS: [&str; 12] = [
    "January", "February", "March", "April", "May", "June", "July", "August",
    "September", "October", "November", "December",
];

const MONTHS_SHORT: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

impl Weekday {
    pub fn name(self) -> &'static str {
        match self {
            Weekday::Monday => "Monday",
            Weekday::Tuesday => "Tuesday",
            Weekday::Wednesday => "Wednesday",
            Weekday::Thursday => "Thursday",
            Weekday::Friday => "Friday",
            Weekday::Saturday => "Saturday",
            Weekday::Sunday => "Sunday",
        }
    }

    pub fn short(self) -> &'static str {
        &self.name()[..3]
    }

    pub fn is_weekend(self) -> bool {
        matches!(self, Weekday::Saturday | Weekday::Sunday)
    }
}

/// Julian Day Number for a proleptic Gregorian date.
pub fn gregorian_to_jdn(year: i32, month: u32, day: u32) -> i32 {
    let a = (14 - month as i32) / 12;
    let y = year + 4800 - a;
    let m = month as i32 + 12 * a - 3;
    day as i32 + (153 * m + 2) / 5 + 365 * y + y / 4 - y / 100 + y / 400 - 32045
}

/// Julian Day Number for a proleptic Julian date.
pub fn julian_to_jdn(year: i32, month: u32, day: u32) -> i32 {
    let a = (14 - month as i32) / 12;
    let y = year + 4800 - a;
    let m = month as i32 + 12 * a - 3;
    day as i32 + (153 * m + 2) / 5 + 365 * y + y / 4 - 32083
}

fn jdn_to_gregorian(jdn: i32) -> (i32, u32, u32) {
    let a = jdn + 32044;
    let b = a.div_euclid(146097);
    let c = a.rem_euclid(146097);
    let d = (4 * c + 3) / 146097;
    let e = c - (146097 * d) / 4;
    let f = (4 * e + 3) / 1461;
    let g = e - (1461 * f) / 4;
    let h = (5 * g + 2) / 153;
    let day = g - (153 * h + 2) / 5 + 1;
    let month = h + 3 - 12 * (h / 10);
    let year = 400 * b + 100 * d + f - 4800 + h / 10;
    (year, month as u32, day as u32)
}

fn jdn_to_julian(jdn: i32) -> (i32, u32, u32) {
    let c = jdn + 32082;
    let d = (4 * c + 3) / 1461;
    let e = c - (1461 * d) / 4;
    let m = (5 * e + 2) / 153;
    let day = e - (153 * m + 2) / 5 + 1;
    let month = m + 3 - 12 * (m / 10);
    let year = d - 4800 + m / 10;
    (year, month as u32, day as u32)
}

impl Date {
    /// The Julian Day Number this date is stored as. The representation is an
    /// implementation detail; this is the supported way to read it.
    pub fn jdn(self) -> i32 {
        self.0
    }

    /// A date from a Julian Day Number, if it lands in the supported years.
    ///
    /// Range-checked on the number itself, because `jdn_to_gregorian` adds a
    /// constant to it and would overflow on the way to finding out.
    pub fn from_jdn(jdn: i32) -> Option<Date> {
        (Date::MIN.0..=Date::MAX.0).contains(&jdn).then_some(Date(jdn))
    }

    pub fn from_ymd(year: i32, month: u32, day: u32) -> Option<Date> {
        if !(MIN_YEAR..=MAX_YEAR).contains(&year)
            || !(1..=12).contains(&month)
            || day == 0
            || day > days_in_month(year, month)
        {
            return None;
        }
        Some(Date(gregorian_to_jdn(year, month, day)))
    }

    /// `days` later, or `None` past the end of the supported years. Ordinary
    /// `+` saturates at the boundary instead, so a walk off the end stops
    /// rather than wrapping into a year the rest of the program cannot hold.
    pub fn checked_add(self, days: i32) -> Option<Date> {
        self.0.checked_add(days).and_then(Date::from_jdn)
    }

    pub fn ymd(self) -> (i32, u32, u32) {
        jdn_to_gregorian(self.0)
    }

    pub fn year(self) -> i32 {
        self.ymd().0
    }

    pub fn month(self) -> u32 {
        self.ymd().1
    }

    pub fn day(self) -> u32 {
        self.ymd().2
    }

    pub fn weekday(self) -> Weekday {
        // JDN 0 was a Monday, so the modulus lands directly on the enum.
        match self.0.rem_euclid(7) {
            0 => Weekday::Monday,
            1 => Weekday::Tuesday,
            2 => Weekday::Wednesday,
            3 => Weekday::Thursday,
            4 => Weekday::Friday,
            5 => Weekday::Saturday,
            _ => Weekday::Sunday,
        }
    }

    /// The Julian calendar date of this civil date.
    pub fn to_julian(self) -> JulianDate {
        let (year, month, day) = jdn_to_julian(self.0);
        JulianDate { year, month, day }
    }

    /// How many days the Julian reckoning currently lags the Gregorian one.
    /// Computed, never assumed, so nothing here breaks at the 2100 boundary.
    pub fn julian_offset(self) -> i32 {
        let j = self.to_julian();
        self.0 - gregorian_to_jdn(j.year, j.month, j.day)
    }

    pub fn iso(self) -> String {
        let (y, m, d) = self.ymd();
        format!("{y:04}-{m:02}-{d:02}")
    }

    /// "Fri 18 Sep", the width the seven-day view lines up on.
    pub fn short(self) -> String {
        let (_, m, d) = self.ymd();
        format!("{} {} {}", self.weekday().short(), d, MONTHS_SHORT[(m - 1) as usize])
    }

    /// "18 September 2026"
    pub fn long(self) -> String {
        let (y, m, d) = self.ymd();
        format!("{} {} {}", d, MONTHS[(m - 1) as usize], y)
    }

    pub fn parse_iso(s: &str) -> Option<Date> {
        let b = s.as_bytes();
        if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
            return None;
        }
        let year = s[0..4].parse().ok()?;
        let month = s[5..7].parse().ok()?;
        let day = s[8..10].parse().ok()?;
        Date::from_ymd(year, month, day)
    }
}

impl JulianDate {
    /// The civil date of a Julian calendar date.
    pub fn to_civil(self) -> Date {
        Date(julian_to_jdn(self.year, self.month, self.day))
    }
}

/// Saturating at the supported range, because a liturgical year is walked by
/// adding days to it and an overflow there would wrap a date into antiquity.
impl std::ops::Add<i32> for Date {
    type Output = Date;
    fn add(self, days: i32) -> Date {
        self.checked_add(days).unwrap_or(if days < 0 { Date::MIN } else { Date::MAX })
    }
}

impl std::ops::Sub<i32> for Date {
    type Output = Date;
    fn sub(self, days: i32) -> Date {
        self.0
            .checked_sub(days)
            .and_then(Date::from_jdn)
            .unwrap_or(if days > 0 { Date::MIN } else { Date::MAX })
    }
}

impl Date {
    /// The first day this calendar answers for.
    pub const MIN: Date = Date(1721426);
    /// The last.
    pub const MAX: Date = Date(5373484);
}

/// Days between two dates, which is a count and not a date.
impl std::ops::Sub<Date> for Date {
    type Output = i32;
    fn sub(self, other: Date) -> i32 {
        self.0 - other.0
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.iso())
    }
}

pub fn is_leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

pub fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap(year) => 29,
        2 => 28,
        _ => 0,
    }
}

/// Is (month, day) within an inclusive fixed-date range, wrapping the year?
///
/// The Nativity Fast does not wrap; the fast-free stretch from the Nativity to
/// the eve of Theophany does. Both are written the same way here.
pub fn fixed_between(md: (u32, u32), start: (u32, u32), end: (u32, u32)) -> bool {
    if start <= end {
        start <= md && md <= end
    } else {
        md >= start || md <= end
    }
}
