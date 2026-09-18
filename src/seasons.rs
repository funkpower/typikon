//! Which season of the year a day falls in.
//!
//! The movable seasons are counted from Pascha and so are compared as civil
//! dates. The fixed ones, the Nativity and Dormition Fasts and the fast-free
//! days after the Nativity, are compared in **ecclesiastical** space, against
//! the date the Church calls the day. That is the whole trick that lets one set
//! of rules serve the old and new calendars at once: the Nativity Fast begins
//! on 15 November in both, and only the mornings differ.
//!
//! The Great Fast here runs from Clean Monday through Palm Sunday. The Forty
//! Days proper end on the Friday before Lazarus Saturday, which is what the
//! fasting rules count from, but a season is a label for a reader, and a Palm
//! Sunday belonging to no season at all is worse than one belonging loosely to
//! the Fast.

use crate::calendars::{fixed_between, Date};
use crate::paschalion::Movables;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Season {
    GreatFast,
    GreatWeek,
    BrightWeek,
    PreLenten,
    CheesefareWeek,
    Apostles,
    Dormition,
    NativityFast,
    PentecostWeek,
    Christmastide,
}

impl Season {
    pub fn name(self) -> &'static str {
        match self {
            Season::GreatFast => "Great Fast",
            Season::GreatWeek => "Great and Holy Week",
            Season::BrightWeek => "Bright Week",
            Season::PreLenten => "Triodion",
            Season::CheesefareWeek => "Cheesefare Week",
            Season::Apostles => "Apostles' Fast",
            Season::Dormition => "Dormition Fast",
            Season::NativityFast => "Nativity Fast",
            Season::PentecostWeek => "Week after Pentecost",
            Season::Christmastide => "Christmastide",
        }
    }

    /// The seasons in which a fast is kept, as distinct from the ones that are
    /// merely named.
    pub fn is_fasting_season(self) -> bool {
        matches!(
            self,
            Season::GreatFast
                | Season::GreatWeek
                | Season::CheesefareWeek
                | Season::Apostles
                | Season::Dormition
                | Season::NativityFast
        )
    }
}

/// The season a day belongs to, or `None` for ordinary time.
///
/// `fixed_md` is the ecclesiastical (month, day). `apostles_end` is the civil
/// date of 28 June reckoned in this jurisdiction's menologion, the eve of Ss
/// Peter and Paul, where the Apostles' Fast stops.
pub fn season_for(
    civil: Date,
    fixed_md: (u32, u32),
    mv: &Movables,
    apostles_end: Date,
) -> Option<Season> {
    // Movable first: Great Week outranks the Great Fast it follows, and Bright
    // Week outranks everything.
    if mv.pascha <= civil && civil <= mv.bright_week_end {
        return Some(Season::BrightWeek);
    }
    if mv.great_week_start <= civil && civil <= mv.holy_saturday {
        return Some(Season::GreatWeek);
    }
    if mv.clean_monday <= civil && civil <= mv.palm_sunday {
        return Some(Season::GreatFast);
    }
    if mv.cheesefare_week_start <= civil && civil <= mv.cheesefare {
        return Some(Season::CheesefareWeek);
    }
    if mv.publican_pharisee <= civil && civil < mv.cheesefare_week_start {
        return Some(Season::PreLenten);
    }
    if mv.pentecost <= civil && civil <= mv.pentecost_week_end {
        return Some(Season::PentecostWeek);
    }
    // The Apostles' Fast can be empty in a year with a late Pascha, which this
    // comparison handles without a special case.
    if mv.apostles_start <= civil && civil <= apostles_end {
        return Some(Season::Apostles);
    }

    // Fixed seasons, in ecclesiastical space.
    if fixed_between(fixed_md, (8, 1), (8, 14)) {
        return Some(Season::Dormition);
    }
    if fixed_between(fixed_md, (11, 15), (12, 24)) {
        return Some(Season::NativityFast);
    }
    if fixed_between(fixed_md, (12, 25), (1, 4)) {
        return Some(Season::Christmastide);
    }
    None
}
