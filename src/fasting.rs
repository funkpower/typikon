//! What is kept on a day, and whether anything about it binds.
//!
//! Six levels, ordered from strictest to laxest. They are the Typikon's own
//! distinctions, not a simplification of them: the difference between a day of
//! xerophagy and a day of wine and oil is the difference the books actually
//! make, and a calendar that reports both as "fasting" has thrown away the
//! information its reader wanted.
//!
//! The same six serve a tradition that makes fewer distinctions. Ruthenian
//! particular law knows three degrees, and each maps exactly onto one of these:
//! its "strict abstinence" is `WineOil`, because it does not regulate oil or
//! wine, and its "abstinence" is `Dairy`, meat alone being set aside. So a
//! ruleset emits the subset it uses and names those levels in its own
//! vocabulary, rather than every tradition being forced through one set of
//! words.
//!
//! **Level and obligation are separate questions.** A day can be kept strictly
//! by universal custom and bind no one, and that is the ordinary case in the
//! Eastern Catholic churches, whose particular law requires very little.

use serde::Serialize;

/// Ordered strictest first, so `a < b` reads as "a is stricter than b" and the
/// derived `Ord` is the comparison the rules want.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Level {
    /// No food until evening.
    Strict,
    /// No oil, no wine; no meat, fish, eggs or dairy.
    Xerophagy,
    /// Wine and oil permitted; no meat, fish, eggs or dairy.
    WineOil,
    /// Fish, wine and oil permitted; no meat, eggs or dairy.
    Fish,
    /// Dairy, eggs and fish permitted; no meat.
    Dairy,
    /// No fast.
    None,
}

impl Level {
    pub fn key(self) -> &'static str {
        match self {
            Level::Strict => "strict",
            Level::Xerophagy => "xerophagy",
            Level::WineOil => "wine_oil",
            Level::Fish => "fish",
            Level::Dairy => "dairy",
            Level::None => "none",
        }
    }

    pub fn is_fast(self) -> bool {
        self != Level::None
    }

    /// The stricter of two levels.
    pub fn stricter_of(self, other: Level) -> Level {
        self.min(other)
    }

    /// The laxer of two levels.
    pub fn laxer_of(self, other: Level) -> Level {
        self.max(other)
    }

    /// Relax towards `target`, but never past `cap` and never tightening.
    ///
    /// This is the arithmetic every "a feast lifts the fast" rule wants, in one
    /// place, so no rule can get the direction backwards.
    pub fn relaxed_to(self, target: Level, cap: Level) -> Level {
        let lifted = self.laxer_of(target).stricter_of(cap);
        if lifted < self {
            self
        } else {
            lifted
        }
    }
}

/// Whether anything about the day actually binds. The Orthodox Church keeps the
/// fast as tradition rather than precept, so its rule reports `Customary`
/// throughout; the Eastern Catholic churches have law, and theirs does not.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Obligation {
    Obligatory,
    Customary,
}

/// How a day is kept, and what about it binds.
#[derive(Clone, Debug)]
pub struct Fast {
    /// The day as the tradition keeps it.
    pub level: Level,
    /// The rule that decided it, so the answer can be argued with.
    pub reason: String,
    pub obligation: Option<Obligation>,
    /// The binding minimum, where one exists. `None` for most jurisdictions,
    /// which is the honest answer rather than a softening.
    pub binds: Level,
}

impl Fast {
    pub fn new(level: Level, reason: &str, obligation: Option<Obligation>) -> Fast {
        Fast {
            level,
            reason: reason.to_string(),
            obligation,
            binds: Level::None,
        }
    }

    /// A day kept by custom, binding no one.
    pub fn customary(level: Level, reason: &str) -> Fast {
        Fast::new(level, reason, Some(Obligation::Customary))
    }

    /// A day with no fast at all.
    pub fn free(reason: &str) -> Fast {
        Fast::new(Level::None, reason, None)
    }

    /// A day that binds, and the minimum it binds to.
    pub fn binding(level: Level, reason: &str, binds: Level) -> Fast {
        Fast {
            level,
            reason: reason.to_string(),
            obligation: Some(Obligation::Obligatory),
            binds,
        }
    }
}
