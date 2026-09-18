//! Fasting disciplines. One module per tradition.

pub mod orthodox;
pub mod ruthenian;

use crate::calendars::{Date, Weekday};
use crate::fasting::{Fast, Level};
use crate::jurisdictions::{Jurisdiction, RuleKind};
use crate::menologion::Rank;
use crate::paschalion::Movables;
use crate::seasons::Season;

/// What a fasting rule is handed. Deliberately small.
pub struct Context<'a> {
    pub civil: Date,
    /// The ecclesiastical (month, day).
    pub fixed: (u32, u32),
    /// The ecclesiastical year, for resolving a fixed date back to a civil one.
    pub fixed_year: i32,
    pub weekday: Weekday,
    pub mv: &'a Movables,
    pub season: Option<Season>,
    pub feast: String,
    pub rank: Option<Rank>,
    pub jurisdiction: &'a Jurisdiction,
}

impl Context<'_> {
    /// The civil date of an ecclesiastical date in this day's church year.
    pub fn civil_of_fixed(&self, month: u32, day: u32) -> Date {
        self.jurisdiction.civil_of_fixed(self.fixed_year, month, day)
    }

    pub fn is_weekend(&self) -> bool {
        self.weekday.is_weekend()
    }

    /// Is this day within `days` of the given start, inclusive?
    pub fn within(&self, start: Date, days: i32) -> bool {
        start <= self.civil && self.civil <= start + days
    }
}

/// How a tradition names the levels it uses, and the glyph it shows them with.
pub struct Vocabulary {
    pub label: fn(Level) -> &'static str,
    pub emoji: fn(Level) -> &'static str,
}

pub struct Rule {
    pub name: &'static str,
    pub title: &'static str,
    pub vocabulary: Vocabulary,
    pub fast_for: fn(&Context) -> Fast,
    /// Whether this tradition legislates the fast as precept at all.
    pub has_obligation: bool,
}

pub fn get(kind: RuleKind) -> &'static Rule {
    match kind {
        RuleKind::Orthodox => &orthodox::RULE,
        RuleKind::Ruthenian => &ruthenian::RULE,
    }
}
