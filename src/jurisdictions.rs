//! Who keeps which calendar, and by whose rule.
//!
//! A jurisdiction is four independent choices, and conflating any two of them
//! is how calendar code usually goes wrong:
//!
//! - **paschalion**: which table gives Pascha
//! - **menologion**: which calendar the fixed feasts are read on
//! - **rule**: which fasting discipline is kept
//! - **menaia**: which supplementary commemorations to load, if any
//!
//! The pairings are not free. Almost every Orthodox Church keeps the Julian
//! Paschalion; what divides them is the menologion. The "old calendar" churches
//! read the fixed feasts on the Julian calendar too, the "new calendar"
//! churches on the Revised Julian, which agrees with the Gregorian for every
//! date until 2800. So a new-calendar church keeps the Nativity with the West
//! and Pascha with the East, which is not a compromise but exactly what the
//! 1923 congress decided.
//!
//! Only two bodies here use the Gregorian Paschalion: the Finnish Orthodox
//! Church, alone among the Orthodox, and the Byzantine Catholic churches.

use crate::calendars::Date;
use crate::paschalion::Paschalion;

/// Which calendar the *fixed* feasts are read on.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MenologionKind {
    Julian,
    /// Revised Julian, which agrees with the Gregorian for every date until 2800.
    RevisedJulian,
}

/// Greek or Slavic usage. Both keep the same Typikon; they read it a little
/// differently, and the differences are small, real and printed in every
/// calendar that takes a side.
///
/// Two are modelled here:
///
/// - **The Protection of the Theotokos**, 1 October, is a great feast in
///   Slavic usage. Greek practice ranks it lower, and the Church of Greece
///   keeps it on 28 October besides.
/// - **Fish in the Nativity Fast.** Greek practice permits fish on any day but
///   Wednesday and Friday until 17 December; Slavic practice keeps fish to
///   Saturdays and Sundays throughout.
///
/// The split is coarse: a church is assigned the usage its books follow, and
/// no jurisdiction is wholly one thing. Override it with `--tradition`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tradition {
    Slavic,
    Greek,
}

impl Tradition {
    pub fn key(self) -> &'static str {
        match self {
            Tradition::Slavic => "slavic",
            Tradition::Greek => "greek",
        }
    }

    pub fn parse(s: &str) -> Option<Tradition> {
        match s.trim().to_ascii_lowercase().as_str() {
            "slavic" | "russian" | "slav" => Some(Tradition::Slavic),
            "greek" | "byzantine" => Some(Tradition::Greek),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RuleKind {
    Orthodox,
    Ruthenian,
}

/// The short name people use for a paschalion/menologion combination.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Calendar {
    Old,
    New,
    Western,
}

impl Calendar {
    pub fn key(self) -> &'static str {
        match self {
            Calendar::Old => "old",
            Calendar::New => "new",
            Calendar::Western => "western",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Jurisdiction {
    pub key: &'static str,
    pub name: &'static str,
    pub paschalion: Paschalion,
    pub menologion: MenologionKind,
    /// Whose particular law decides what binds, and, unless `discipline`
    /// says otherwise, at what level the day is shown.
    pub rule: RuleKind,
    /// Show the day at another tradition's discipline while still reporting
    /// this church's obligations.
    ///
    /// The Eastern Catholic churches are the case this exists for. Their
    /// tradition is the Byzantine fast entire; their law requires very much
    /// less of it. Reporting only the law makes the Great Fast look like four
    /// days of abstinence a year, which is what binds and not what is kept.
    pub discipline: Option<RuleKind>,
    pub tradition: Tradition,
    /// Supplementary commemoration sets to overlay, fetched not shipped.
    pub menaia: &'static [&'static str],
    pub note: &'static str,
}

impl Jurisdiction {
    pub fn calendar(&self) -> Calendar {
        match (self.paschalion, self.menologion) {
            (Paschalion::Gregorian, MenologionKind::RevisedJulian) => Calendar::Western,
            (_, MenologionKind::Julian) => Calendar::Old,
            _ => Calendar::New,
        }
    }

    /// The ecclesiastical (year, month, day) a civil date is reckoned as.
    pub fn fixed_of_civil(&self, date: Date) -> (i32, u32, u32) {
        match self.menologion {
            MenologionKind::Julian => {
                let j = date.to_julian();
                (j.year, j.month, j.day)
            }
            MenologionKind::RevisedJulian => date.ymd(),
        }
    }

    /// The civil date of an ecclesiastical date in a given church year.
    pub fn civil_of_fixed(&self, year: i32, month: u32, day: u32) -> Date {
        match self.menologion {
            MenologionKind::Julian => {
                crate::calendars::JulianDate { year, month, day }.to_civil()
            }
            MenologionKind::RevisedJulian => Date::from_ymd(year, month, day)
                .unwrap_or_else(|| Date::from_ymd(year, month, 28).expect("28th exists")),
        }
    }
}

use MenologionKind::{Julian as JulMen, RevisedJulian as RevJul};
use Paschalion::{Gregorian as GregP, Julian as JulP};
use RuleKind::{Orthodox as Orth, Ruthenian as Ruth};
use Tradition::{Greek as Gk, Slavic as Sl};

const NO_MENAIA: &[&str] = &[];
const RUTHENIAN_MENAIA: &[&str] = &["ruthenian"];

/// A reckoning without naming a church. `new` is the default.
pub const CALENDARS: &[Jurisdiction] = &[
    Jurisdiction { key: "old", name: "Old Calendar", paschalion: JulP, menologion: JulMen, rule: Orth, discipline: None, tradition: Sl, menaia: NO_MENAIA,
        note: "Julian throughout, the reckoning of Moscow, Serbia, Georgia, Jerusalem and the Holy Mountain." },
    Jurisdiction { key: "new", name: "New Calendar", paschalion: JulP, menologion: RevJul, rule: Orth, discipline: None, tradition: Gk, menaia: NO_MENAIA,
        note: "Revised Julian fixed feasts with the Julian Paschalion: Constantinople, Greece, Antioch, Romania, Bulgaria, the OCA." },
    Jurisdiction { key: "western", name: "Gregorian", paschalion: GregP, menologion: RevJul, rule: Orth, discipline: None, tradition: Gk, menaia: NO_MENAIA,
        note: "Gregorian throughout, Pascha included. Kept by the Finnish Orthodox Church and by most Byzantine Catholics." },
];

pub const JURISDICTIONS: &[Jurisdiction] = &[
    // old calendar
    Jurisdiction { key: "moscow", name: "Russian Orthodox Church", paschalion: JulP, menologion: JulMen, rule: Orth, discipline: None, tradition: Sl, menaia: NO_MENAIA, note: "" },
    Jurisdiction { key: "rocor", name: "Russian Orthodox Church Outside Russia", paschalion: JulP, menologion: JulMen, rule: Orth, discipline: None, tradition: Sl, menaia: NO_MENAIA, note: "" },
    Jurisdiction { key: "serbian", name: "Serbian Orthodox Church", paschalion: JulP, menologion: JulMen, rule: Orth, discipline: None, tradition: Sl, menaia: NO_MENAIA, note: "" },
    Jurisdiction { key: "georgian", name: "Georgian Orthodox Church", paschalion: JulP, menologion: JulMen, rule: Orth, discipline: None, tradition: Gk, menaia: NO_MENAIA, note: "" },
    Jurisdiction { key: "jerusalem", name: "Patriarchate of Jerusalem", paschalion: JulP, menologion: JulMen, rule: Orth, discipline: None, tradition: Gk, menaia: NO_MENAIA, note: "" },
    Jurisdiction { key: "athos", name: "The Holy Mountain", paschalion: JulP, menologion: JulMen, rule: Orth, discipline: None, tradition: Gk, menaia: NO_MENAIA,
        note: "Athonite practice is stricter than the printed Typikon at many points." },
    Jurisdiction { key: "polish", name: "Polish Orthodox Church", paschalion: JulP, menologion: JulMen, rule: Orth, discipline: None, tradition: Sl, menaia: NO_MENAIA,
        note: "Returned to the Julian menologion in 2014 after decades on the Revised Julian; some parishes kept the new calendar." },
    Jurisdiction { key: "macedonian", name: "Macedonian Orthodox Church", paschalion: JulP, menologion: JulMen, rule: Orth, discipline: None, tradition: Sl, menaia: NO_MENAIA, note: "" },
    Jurisdiction { key: "acrod", name: "American Carpatho-Russian Orthodox Diocese", paschalion: JulP, menologion: JulMen, rule: Orth, discipline: None, tradition: Sl, menaia: NO_MENAIA,
        note: "The Carpatho-Rusyn diocese under Constantinople, formed from the Greek Catholic parishes that returned to Orthodoxy in 1938. The diocese publishes both an old-calendar and a new-calendar cycle and its parishes keep one or the other; this entry is the old one, and `--calendar new --tradition slavic` is the other." },

    // new calendar
    Jurisdiction { key: "constantinople", name: "Ecumenical Patriarchate", paschalion: JulP, menologion: RevJul, rule: Orth, discipline: None, tradition: Gk, menaia: NO_MENAIA, note: "" },
    Jurisdiction { key: "goarch", name: "Greek Orthodox Archdiocese of America", paschalion: JulP, menologion: RevJul, rule: Orth, discipline: None, tradition: Gk, menaia: NO_MENAIA, note: "" },
    Jurisdiction { key: "greece", name: "Church of Greece", paschalion: JulP, menologion: RevJul, rule: Orth, discipline: None, tradition: Gk, menaia: NO_MENAIA, note: "" },
    Jurisdiction { key: "antiochian", name: "Antiochian Orthodox Church", paschalion: JulP, menologion: RevJul, rule: Orth, discipline: None, tradition: Gk, menaia: NO_MENAIA, note: "" },
    Jurisdiction { key: "oca", name: "Orthodox Church in America", paschalion: JulP, menologion: RevJul, rule: Orth, discipline: None, tradition: Sl, menaia: NO_MENAIA,
        note: "Most parishes keep the new calendar; a minority, chiefly of Carpatho-Russian and Alaskan background, keep the old." },
    Jurisdiction { key: "romanian", name: "Romanian Orthodox Church", paschalion: JulP, menologion: RevJul, rule: Orth, discipline: None, tradition: Gk, menaia: NO_MENAIA, note: "" },
    Jurisdiction { key: "bulgarian", name: "Bulgarian Orthodox Church", paschalion: JulP, menologion: RevJul, rule: Orth, discipline: None, tradition: Sl, menaia: NO_MENAIA, note: "" },
    Jurisdiction { key: "cyprus", name: "Church of Cyprus", paschalion: JulP, menologion: RevJul, rule: Orth, discipline: None, tradition: Gk, menaia: NO_MENAIA, note: "" },
    Jurisdiction { key: "albanian", name: "Orthodox Church of Albania", paschalion: JulP, menologion: RevJul, rule: Orth, discipline: None, tradition: Gk, menaia: NO_MENAIA, note: "" },
    Jurisdiction { key: "ukrainian", name: "Orthodox Church of Ukraine", paschalion: JulP, menologion: RevJul, rule: Orth, discipline: None, tradition: Sl, menaia: NO_MENAIA,
        note: "Moved the fixed feasts to the Revised Julian calendar on 1 September 2023. The Julian Paschalion is retained." },

    // gregorian throughout
    Jurisdiction { key: "finnish", name: "Finnish Orthodox Church", paschalion: GregP, menologion: RevJul, rule: Orth, discipline: None, tradition: Gk, menaia: NO_MENAIA,
        note: "The only Orthodox Church keeping the Gregorian Paschalion." },

    // eastern catholic
    Jurisdiction { key: "ruthenian", name: "Byzantine Catholic Metropolia of Pittsburgh", paschalion: GregP, menologion: RevJul, rule: Ruth, discipline: None, tradition: Sl, menaia: RUTHENIAN_MENAIA,
        note: "Gregorian throughout. Fasting is reported under Ruthenian particular law, which binds far less than the Typikon prescribes." },
    Jurisdiction { key: "ugcc", name: "Ukrainian Greek Catholic Church", paschalion: JulP, menologion: RevJul, rule: Ruth, discipline: None, tradition: Sl, menaia: RUTHENIAN_MENAIA,
        note: "Moved the fixed feasts to the Revised Julian calendar on 1 September 2023, keeping the Julian Paschalion. Eparchies outside Ukraine vary; several keep the Gregorian Paschalion." },
    Jurisdiction { key: "melkite", name: "Melkite Greek Catholic Church", paschalion: GregP, menologion: RevJul, rule: Ruth, discipline: None, tradition: Gk, menaia: NO_MENAIA,
        note: "Practice is not uniform: parishes in the Middle East commonly keep the Julian Paschalion for the sake of the local Orthodox." },
    Jurisdiction { key: "romanian-catholic", name: "Romanian Greek Catholic Church", paschalion: GregP, menologion: RevJul, rule: Ruth, discipline: None, tradition: Gk, menaia: NO_MENAIA, note: "" },
];

/// A jurisdiction key, a calendar name, or `None` for the default.
pub fn resolve(name: Option<&str>) -> Result<&'static Jurisdiction, String> {
    let Some(name) = name else {
        return Ok(&CALENDARS[1]); // new
    };
    let key = name.trim().to_ascii_lowercase();
    JURISDICTIONS
        .iter()
        .chain(CALENDARS.iter())
        .find(|j| j.key == key)
        .ok_or_else(|| name.to_string())
}
