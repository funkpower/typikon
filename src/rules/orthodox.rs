//! The Typikon's fast, as the Orthodox Church keeps it.
//!
//! This is the discipline printed in the Typikon and reproduced, with small
//! variations, by every Orthodox jurisdiction's published calendar. It binds no
//! one in the canonical sense: the Orthodox Church legislates fasting as
//! tradition rather than as precept, so every day here is reported as
//! customary. That is not a softening. It is the difference between a rule of
//! life and a law, and the Eastern Catholic churches, which do have law about
//! this, are handled in their own module.
//!
//! A saint of vigil or polyeleos rank lifts the weekday discipline of a fast:
//! to wine and oil inside the Great Fast, as far as fish in the three lesser
//! fasts. That is why the printed calendars mark so many Wednesdays in the
//! Nativity Fast for wine and oil, and it is the rank in the menologion that
//! decides it.
//!
//! Where usages differ, this follows the common published rule and says so in
//! the reason. The known divergences:
//!
//! - **Great Thursday** is given wine and oil here. Some typika keep xerophagy.
//! - **The Exaltation of the Cross** and the **Beheading of the Forerunner**
//!   are given wine and oil. Several jurisdictions print them as strict.
//! - **Lazarus Saturday** is given wine and oil; caviar is the traditional
//!   addition, which this vocabulary has no level for.
//! - **Great Week** is kept at xerophagy on its weekdays whatever saint falls
//!   in it, because the days of the Passion outrank the menologion.
//! - **Fish in the Nativity Fast** follows the usage, Greek or Slavic. See
//!   `Tradition`.
//! - **Monastic practice**, and the Holy Mountain in particular, is stricter
//!   than any of this throughout.
//!
//! A calendar cannot settle those. Your bishop and your spiritual father can.

use super::{Context, Rule, Vocabulary};
use crate::calendars::Weekday;
use crate::fasting::{Fast, Level};
use crate::jurisdictions::Tradition;
use crate::menologion::Rank;
use crate::seasons::Season;

fn label(level: Level) -> &'static str {
    match level {
        Level::Strict => "no food until evening",
        Level::Xerophagy => "xerophagy \u{2014} no oil or wine",
        Level::WineOil => "wine and oil permitted",
        Level::Fish => "fish, wine and oil",
        Level::Dairy => "dairy and eggs; no meat",
        Level::None => "no fast",
    }
}

fn emoji(level: Level) -> &'static str {
    match level {
        Level::Strict => "\u{26d4}",
        Level::Xerophagy => "\u{1f96c}",
        Level::WineOil => "\u{1fad2}",
        Level::Fish => "\u{1f41f}",
        Level::Dairy => "\u{1f9c0}",
        Level::None => "\u{1f356}",
    }
}

/// How far a feast lifts a fast day.
fn relaxed_by(rank: Option<Rank>) -> Option<Level> {
    match rank {
        Some(Rank::Great) => Some(Level::Fish),
        Some(Rank::Vigil) | Some(Rank::Polyeleos) => Some(Level::WineOil),
        None => None,
    }
}

/// Relax `level` for the day's saint, never past `cap`, never tightening.
fn for_feast(level: Level, ctx: &Context, cap: Level, reason: &str) -> Fast {
    let Some(target) = relaxed_by(ctx.rank) else {
        return Fast::customary(level, reason);
    };
    let lifted = level.relaxed_to(target, cap);
    if lifted == level {
        return Fast::customary(level, reason);
    }
    Fast::customary(lifted, &format!("{} \u{2014} the fast is relaxed", ctx.feast))
}

pub fn fast_for(ctx: &Context) -> Fast {
    let md = ctx.fixed;
    let mv = ctx.mv;
    let weekend = ctx.is_weekend();

    // The fast-free stretches, settled before anything else can override them.
    // A Friday inside Bright Week is not a fast day, and the ordinary Friday
    // rule further down would say otherwise.
    match ctx.season {
        Some(Season::BrightWeek) => return Fast::free("Bright Week \u{2014} no fasting"),
        Some(Season::PentecostWeek) => {
            return Fast::free("the week after Pentecost \u{2014} no fasting")
        }
        Some(Season::Christmastide) => {
            return Fast::free("the Nativity to the eve of Theophany")
        }
        _ => {}
    }
    if ctx.within(mv.publican_pharisee, 6) {
        // Fast-free precisely because the Pharisee boasted of fasting twice a week.
        return Fast::free("fast-free week (the Publican and the Pharisee)");
    }

    // The Great Fast and Great Week.
    if matches!(ctx.season, Some(Season::GreatFast) | Some(Season::GreatWeek)) {
        let season = ctx.season.unwrap();
        // The Annunciation is tested before the strict days, not after. It can
        // land on Great Friday or Great Saturday, and when it does the feast
        // still lifts the total fast to wine and oil: it does not vanish.
        if md == (3, 25) {
            if ctx.civil == mv.great_friday || ctx.civil == mv.holy_saturday {
                return Fast::customary(
                    Level::WineOil,
                    "the Annunciation, on a day of the total fast",
                );
            }
            return Fast::customary(Level::Fish, "the Annunciation");
        }
        if ctx.civil == mv.palm_sunday {
            return Fast::customary(Level::Fish, "Palm Sunday");
        }
        if ctx.civil == mv.lazarus_saturday {
            return Fast::customary(Level::WineOil, "Lazarus Saturday");
        }
        if ctx.civil == mv.clean_monday {
            return Fast::customary(Level::Strict, "the first day of the Great Fast");
        }
        if ctx.civil == mv.great_friday {
            return Fast::customary(Level::Strict, "Great and Holy Friday");
        }
        if ctx.civil == mv.holy_saturday {
            // The one Saturday of the year kept without oil.
            return Fast::customary(Level::Xerophagy, "Great and Holy Saturday");
        }
        if ctx.civil == mv.great_thursday {
            return Fast::customary(Level::WineOil, "Great and Holy Thursday");
        }
        if season == Season::GreatWeek {
            return Fast::customary(
                Level::Xerophagy,
                &format!("a weekday of the {}", season.name()),
            );
        }
        if weekend {
            return Fast::customary(
                Level::WineOil,
                &format!("Saturday and Sunday of the {}", season.name()),
            );
        }
        return for_feast(
            Level::Xerophagy,
            ctx,
            Level::WineOil,
            &format!("a weekday of the {}", season.name()),
        );
    }

    // Cheesefare: meat is set aside for good, everything else stays,
    // Wednesday and Friday included.
    if ctx.season == Some(Season::CheesefareWeek) {
        return Fast::customary(Level::Dairy, "Cheesefare Week");
    }
    if ctx.civil == mv.meatfare {
        return Fast::free("Meatfare Sunday \u{2014} the last day of meat");
    }

    // The Nativity Fast, which tightens for its last five days.
    if ctx.season == Some(Season::NativityFast) {
        if md == (12, 24) {
            // The eve is relaxed to wine and oil on a Saturday or Sunday,
            // because the Liturgy of St Basil is served with Vespers.
            if weekend {
                return Fast::customary(Level::WineOil, "the eve of the Nativity, on a weekend");
            }
            return Fast::customary(Level::Xerophagy, "the eve of the Nativity");
        }
        if md == (11, 21) {
            return Fast::customary(Level::Fish, "the Entrance of the Theotokos");
        }
        if md >= (12, 20) {
            if matches!(ctx.weekday, Weekday::Tuesday | Weekday::Thursday) || weekend {
                return Fast::customary(Level::WineOil, "the forefeast of the Nativity");
            }
            return for_feast(Level::Xerophagy, ctx, Level::WineOil, "the forefeast of the Nativity");
        }
        // Greek practice permits fish on any day but Wednesday and Friday
        // until 17 December; Slavic practice keeps it to Saturdays and
        // Sundays. This is the one place the two usages part on the fast.
        if ctx.tradition == Tradition::Greek && md <= (12, 17) {
            if matches!(ctx.weekday, Weekday::Wednesday | Weekday::Friday) {
                return for_feast(Level::Xerophagy, ctx, Level::Fish,
                                 "Wednesday and Friday of the Nativity Fast");
            }
            return Fast::customary(Level::Fish, "the Nativity Fast, Greek usage");
        }
        if weekend {
            return Fast::customary(Level::Fish, "Saturday and Sunday of the Nativity Fast");
        }
        if matches!(ctx.weekday, Weekday::Tuesday | Weekday::Thursday) {
            return for_feast(Level::WineOil, ctx, Level::Fish, "Tuesday and Thursday of the Nativity Fast");
        }
        return for_feast(Level::Xerophagy, ctx, Level::Fish, "Monday, Wednesday and Friday of the Nativity Fast");
    }

    // The Apostles' Fast, kept the same way.
    if ctx.season == Some(Season::Apostles) {
        if weekend {
            return Fast::customary(Level::Fish, "Saturday and Sunday of the Apostles' Fast");
        }
        if matches!(ctx.weekday, Weekday::Tuesday | Weekday::Thursday) {
            return for_feast(Level::WineOil, ctx, Level::Fish, "Tuesday and Thursday of the Apostles' Fast");
        }
        return for_feast(Level::Xerophagy, ctx, Level::Fish, "Monday, Wednesday and Friday of the Apostles' Fast");
    }

    // The Dormition Fast, kept like the Great Fast.
    if ctx.season == Some(Season::Dormition) {
        if md == (8, 6) {
            return Fast::customary(Level::Fish, "the Transfiguration");
        }
        if weekend {
            return Fast::customary(Level::WineOil, "Saturday and Sunday of the Dormition Fast");
        }
        return for_feast(Level::Xerophagy, ctx, Level::WineOil, "a weekday of the Dormition Fast");
    }

    // The three fixed fast days outside any season.
    if md == (1, 5) {
        if weekend {
            return Fast::customary(Level::WineOil, "the eve of Theophany, on a weekend");
        }
        return Fast::customary(Level::Xerophagy, "the eve of Theophany");
    }
    if md == (9, 14) {
        return Fast::customary(Level::WineOil, "the Exaltation of the Holy Cross");
    }
    if md == (8, 29) {
        return Fast::customary(Level::WineOil, "the Beheading of the Forerunner");
    }

    // Ordinary Wednesdays and Fridays, which a feast can lift.
    if matches!(ctx.weekday, Weekday::Wednesday | Weekday::Friday) {
        if let Some(target) = relaxed_by(ctx.rank) {
            return Fast::customary(target, &format!("{} \u{2014} the fast is relaxed", ctx.feast));
        }
        return Fast::customary(Level::Xerophagy, "Wednesday and Friday");
    }

    Fast::new(Level::None, "", None)
}

pub static RULE: Rule = Rule {
    name: "orthodox",
    title: "the Typikon",
    vocabulary: Vocabulary { label, emoji },
    fast_for,
    obligation_for: None,
    // The Orthodox Church does not legislate the fast as precept, so nothing
    // here is reported as binding. See the module docs.
    has_obligation: false,
};
