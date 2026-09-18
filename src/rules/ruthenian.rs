//! Byzantine Catholic particular law, alongside the tradition it sits on.
//!
//! The Eastern Catholic churches differ from the Orthodox here in a way worth
//! being exact about: they have *law* on fasting, and it requires very much
//! less than the Typikon describes. Under current Ruthenian particular law the
//! Church binds only:
//!
//! - strict abstinence, no meat, fish, eggs or dairy, on the first day of the
//!   Great Fast and on Great and Holy Friday;
//! - abstinence from meat on the Wednesdays and Fridays of the Great Fast.
//!
//! Everything else, the Apostles', Dormition and Nativity Fasts, the fuller
//! keeping of the Great Fast, ordinary Fridays through the year, is encouraged
//! and traditional and does **not** bind. So this module reports two things at
//! once: the level at which the day is kept by tradition, and the minimum that
//! actually obliges. A calendar that says "obligatory" where nothing obliges is
//! worse than no calendar; one that reports only the binding minimum makes the
//! obligatory days look laxer than the voluntary ones beside them.
//!
//! The three degrees this law knows map exactly onto the shared vocabulary: its
//! strict abstinence is `WineOil`, since it does not regulate oil or wine, and
//! its simple abstinence is `Dairy`, meat alone being set aside.
//!
//! **A known reading.** "The Wednesdays and Fridays of the Great Fast" is taken
//! here to mean the Forty Days, not Great and Holy Week, which the calendar
//! counts as its own season. Great and Holy Friday binds in its own right, so
//! only Great and Holy Wednesday turns on the reading. If your eparchy reads it
//! the other way, that Wednesday binds too.
//!
//! Source: the Byzantine Catholic Metropolitan Church of Pittsburgh. Other
//! Eastern Catholic churches legislate their own; where this library maps them
//! onto this ruleset it is an approximation, and the jurisdiction entry says so.

use super::{Context, Rule, Vocabulary};
use crate::calendars::{Date, Weekday};
use crate::fasting::{Fast, Level};
use crate::menologion::Rank;
use crate::seasons::Season;

fn label(level: Level) -> &'static str {
    match level {
        Level::WineOil => "no meat, fish, eggs or dairy",
        Level::Dairy => "no meat",
        Level::None => "no fast",
        // Levels this law makes no distinction at, kept legible if ever emitted.
        Level::Strict => "no food until evening",
        Level::Xerophagy => "no meat, fish, eggs or dairy",
        Level::Fish => "no meat",
    }
}

fn emoji(level: Level) -> &'static str {
    match level {
        Level::Strict | Level::Xerophagy | Level::WineOil => "\u{1f96c}",
        Level::Fish | Level::Dairy => "\u{1f41f}",
        Level::None => "\u{1f356}",
    }
}

/// Vigil fasts are kept on the preceding Friday when they land on a weekend.
///
/// "If the vigils of Christmas or Theophany fall on Saturday or Sunday, the
/// fasting and abstinence is done on the previous Friday."
fn off_weekend(day: Date) -> Date {
    match day.weekday() {
        Weekday::Saturday => day - 1,
        Weekday::Sunday => day - 2,
        _ => day,
    }
}

pub fn fast_for(ctx: &Context) -> Fast {
    let mv = ctx.mv;

    // The two days the Church binds strictly.
    if ctx.civil == mv.clean_monday {
        return Fast::binding(Level::WineOil, "the first day of the Great Fast", Level::WineOil);
    }
    if ctx.civil == mv.great_friday {
        return Fast::binding(Level::WineOil, "Great and Holy Friday", Level::WineOil);
    }

    // And the days it binds to abstinence from meat. Kept strictly by
    // tradition; binding only as far as meat.
    if ctx.season == Some(Season::GreatFast)
        && matches!(ctx.weekday, Weekday::Wednesday | Weekday::Friday)
    {
        return Fast::binding(
            Level::WineOil,
            "Wednesday and Friday of the Great Fast",
            Level::Dairy,
        );
    }

    // Fixed vigils and the Exaltation, moved off a weekend where the rule says.
    for (when, label) in [
        (ctx.civil_of_fixed(9, 14), "the Exaltation of the Holy Cross"),
        (off_weekend(ctx.civil_of_fixed(12, 24)), "the vigil of the Nativity"),
        (off_weekend(ctx.civil_of_fixed(1, 5)), "the vigil of Theophany"),
    ] {
        if ctx.civil == when {
            return Fast::customary(Level::WineOil, label);
        }
    }

    // The fast-free stretches, which have to be settled before the ordinary
    // Wednesday and Friday rule below or they get overridden by it. Christmas
    // Day 2026 is a Friday, and telling you to abstain from meat on it would be
    // plainly wrong.
    match ctx.season {
        Some(Season::BrightWeek) => return Fast::free("Bright Week \u{2014} no fasting"),
        Some(Season::PentecostWeek) => {
            return Fast::free("the week after Pentecost \u{2014} no fasting")
        }
        Some(Season::Christmastide) => return Fast::free("Christmastide \u{2014} no fasting"),
        _ => {}
    }
    if ctx.within(mv.publican_pharisee, 6) {
        return Fast::free("fast-free week (the Publican and the Pharisee)");
    }

    // The rest of the Great Fast and Great Week.
    if let Some(season @ (Season::GreatFast | Season::GreatWeek)) = ctx.season {
        if ctx.is_weekend() {
            return Fast::customary(Level::Dairy, season.name());
        }
        return Fast::customary(Level::WineOil, season.name());
    }

    if ctx.season == Some(Season::CheesefareWeek) {
        return Fast::customary(Level::Dairy, "Cheesefare Week");
    }

    // The three voluntary seasons.
    if let Some(season @ (Season::Apostles | Season::Dormition | Season::NativityFast)) =
        ctx.season
    {
        return Fast::customary(Level::Dairy, season.name());
    }

    // A great feast releases the ordinary weekday abstinence.
    if matches!(ctx.weekday, Weekday::Wednesday | Weekday::Friday) {
        if ctx.rank == Some(Rank::Great) {
            return Fast::free(&format!("{} \u{2014} the fast is released", ctx.feast));
        }
        return Fast::customary(
            Level::Dairy,
            if ctx.weekday == Weekday::Friday { "Friday" } else { "Wednesday" },
        );
    }

    Fast::new(Level::None, "", None)
}

pub static RULE: Rule = Rule {
    name: "ruthenian",
    title: "Ruthenian particular law",
    vocabulary: Vocabulary { label, emoji },
    fast_for,
    has_obligation: true,
};
