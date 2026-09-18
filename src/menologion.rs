//! The fixed calendar, and the movable feasts that outrank it.
//!
//! The menologion ships once, compiled into the binary, and serves every
//! jurisdiction because it is indexed by **ecclesiastical** date. A church on
//! the old calendar and one on the new both keep the Nativity on 25 December;
//! they disagree about which morning that is, and that disagreement lives in
//! the calendar conversion, not here.
//!
//! Supplementary sets, a local church's own saints, are fetched rather than
//! shipped, because they are usually published by a body that holds the
//! copyright in the wording. They overlay the common set at runtime.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::calendars::Date;
use crate::jurisdictions::Tradition;
use crate::paschalion::Movables;

/// The degrees of solemnity the menologion marks, and the order they rank in.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Rank {
    Polyeleos,
    Vigil,
    Great,
}

impl Rank {
    pub fn key(self) -> &'static str {
        match self {
            Rank::Polyeleos => "polyeleos",
            Rank::Vigil => "vigil",
            Rank::Great => "great",
        }
    }

    /// Whether a feast of this rank names the day, as against merely being
    /// commemorated on it.
    pub fn names_the_day(self) -> bool {
        matches!(self, Rank::Great | Rank::Vigil)
    }

    /// The mark the liturgical books print beside the day.
    ///
    /// In the Slavic books these are printed in red: a cross in a circle for a
    /// great feast, a plain cross for a vigil, a cross over a half-circle for
    /// a polyeleos. Unicode has no glyph for the last, so the Jerusalem cross
    /// stands in for it, which is what digital calendars generally settle on.
    pub fn sign(self) -> &'static str {
        match self {
            Rank::Great => "\u{2295}",
            Rank::Vigil => "\u{271a}",
            Rank::Polyeleos => "\u{2629}",
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Entry {
    #[serde(default)]
    pub commemorations: Vec<String>,
    #[serde(default)]
    pub rank: Option<Rank>,
    /// Forefeast, afterfeast or leave-taking.
    #[serde(default)]
    pub cycle: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OverlayMode {
    /// The set's commemorations join the common ones. Right for a local set
    /// that adds a few saints of its own.
    #[default]
    Add,
    /// The set's commemorations stand in for them, because it is itself a
    /// complete calendar published by that church.
    Replace,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct Meta {
    #[serde(default)]
    mode: OverlayMode,
}

#[derive(Clone, Debug, Default)]
pub struct Menologion {
    entries: BTreeMap<String, Entry>,
    mode: OverlayMode,
}

/// The common Byzantine menologion, compiled in. No file to find, no path to
/// get wrong, and the binary answers for every day of the year on its own.
const COMMON: &str = include_str!("../data/menologion.json");

/// Where the two usages part company. Compiled in beside the common set, and
/// laid over it according to the jurisdiction's tradition.
const SLAVIC: &str = include_str!("../data/tradition-slavic.json");
const GREEK: &str = include_str!("../data/tradition-greek.json");

fn parse(text: &str) -> Result<Menologion, String> {
    let raw: BTreeMap<String, serde_json::Value> =
        serde_json::from_str(text).map_err(|e| e.to_string())?;
    let mut entries = BTreeMap::new();
    let mut mode = OverlayMode::Add;
    for (key, value) in raw {
        if key == "_meta" {
            let meta: Meta = serde_json::from_value(value).unwrap_or_default();
            mode = meta.mode;
            continue;
        }
        let entry: Entry = serde_json::from_value(value).map_err(|e| format!("{key}: {e}"))?;
        entries.insert(key, entry);
    }
    Ok(Menologion { entries, mode })
}

impl Menologion {
    pub fn common() -> &'static Menologion {
        use std::sync::OnceLock;
        static ONCE: OnceLock<Menologion> = OnceLock::new();
        ONCE.get_or_init(|| parse(COMMON).expect("the shipped menologion parses"))
    }

    pub fn get(&self, month: u32, day: u32) -> Option<&Entry> {
        self.entries.get(&format!("{month:02}-{day:02}"))
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Lay another set over this one. Rank and cycle survive from the common
    /// set unless the overlay gives its own, since a published list of saints
    /// usually carries neither and the fasting rules need the rank.
    pub fn overlay(&self, other: &Menologion) -> Menologion {
        let mut merged = self.entries.clone();
        for (key, add) in &other.entries {
            let base = merged.entry(key.clone()).or_default();
            match other.mode {
                OverlayMode::Replace => {
                    if !add.commemorations.is_empty() {
                        base.commemorations = add.commemorations.clone();
                    }
                }
                OverlayMode::Add => {
                    for c in &add.commemorations {
                        if !base.commemorations.contains(c) {
                            base.commemorations.push(c.clone());
                        }
                    }
                }
            }
            if add.rank.is_some() {
                base.rank = add.rank;
            }
            if add.cycle.is_some() {
                base.cycle = add.cycle.clone();
            }
        }
        Menologion { entries: merged, mode: self.mode }
    }

    pub fn load_file(path: &std::path::Path) -> Result<Menologion, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        parse(&text)
    }
}

/// Where fetched supplementary sets live.
pub fn data_dir() -> std::path::PathBuf {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            let home = std::env::var_os("HOME").unwrap_or_default();
            std::path::Path::new(&home).join(".local/share")
        });
    base.join("typikon")
}

fn tradition_overlay(t: Tradition) -> &'static Menologion {
    use std::sync::OnceLock;
    static SL: OnceLock<Menologion> = OnceLock::new();
    static GK: OnceLock<Menologion> = OnceLock::new();
    match t {
        Tradition::Slavic => SL.get_or_init(|| parse(SLAVIC).expect("slavic usage parses")),
        Tradition::Greek => GK.get_or_init(|| parse(GREEK).expect("greek usage parses")),
    }
}

/// The common menologion, the jurisdiction's usage laid over it, then any
/// supplementary sets. A missing fetched set is not an error: it simply has
/// not been fetched.
pub fn for_jurisdiction(tradition: Tradition, menaia: &[&str]) -> Menologion {
    let mut m = Menologion::common().overlay(tradition_overlay(tradition));
    for name in menaia {
        let path = data_dir().join(format!("{name}.json"));
        if let Ok(extra) = Menologion::load_file(&path) {
            m = m.overlay(&extra);
        }
    }
    m
}

/// The movable feasts, named by their offset from Pascha. These outrank
/// whatever the fixed calendar holds for the day.
pub fn movable_feast_for(civil: Date, mv: &Movables) -> Option<(&'static str, Option<Rank>)> {
    let table: [(Date, &'static str, Option<Rank>); 24] = [
        (mv.pascha, "The Radiant and Life-giving Resurrection of our Lord \u{2014} Holy Pascha", Some(Rank::Great)),
        (mv.palm_sunday, "The Entry of our Lord into Jerusalem", Some(Rank::Great)),
        (mv.ascension, "The Ascension of our Lord, God and Saviour Jesus Christ", Some(Rank::Great)),
        (mv.pentecost, "Holy Pentecost \u{2014} the Descent of the Holy Spirit", Some(Rank::Great)),
        (mv.lazarus_saturday, "Lazarus Saturday", Some(Rank::Vigil)),
        (mv.great_thursday, "Great and Holy Thursday", Some(Rank::Vigil)),
        (mv.great_friday, "Great and Holy Friday", Some(Rank::Vigil)),
        (mv.holy_saturday, "Great and Holy Saturday", Some(Rank::Vigil)),
        (mv.thomas_sunday, "Thomas Sunday \u{2014} the Antipascha", Some(Rank::Polyeleos)),
        (mv.myrrhbearers, "The Sunday of the Holy Myrrh-bearing Women", Some(Rank::Polyeleos)),
        (mv.midfeast, "The Midfeast of Pentecost", Some(Rank::Polyeleos)),
        (mv.all_saints, "The Sunday of All Saints", Some(Rank::Polyeleos)),
        (mv.publican_pharisee, "The Sunday of the Publican and the Pharisee", None),
        (mv.prodigal_son, "The Sunday of the Prodigal Son", None),
        (mv.meatfare, "Meatfare Sunday \u{2014} the Sunday of the Last Judgement", None),
        (mv.cheesefare, "Cheesefare Sunday \u{2014} the Expulsion of Adam, Forgiveness Sunday", None),
        (mv.orthodoxy_sunday, "The Sunday of Orthodoxy", Some(Rank::Polyeleos)),
        (mv.gregory_palamas, "The Sunday of St Gregory Palamas", None),
        (mv.cross_veneration, "The Sunday of the Veneration of the Cross", Some(Rank::Polyeleos)),
        (mv.john_climacus, "The Sunday of St John Climacus", None),
        (mv.mary_of_egypt, "The Sunday of St Mary of Egypt", None),
        (mv.fathers_first_council, "The Sunday of the Fathers of the First Council", None),
        (mv.clean_monday, "Clean Monday \u{2014} the first day of the Great Fast", None),
        // A sentinel that can never match, keeping the array a fixed size.
        (Date(i32::MIN), "", None),
    ];
    table
        .iter()
        .find(|(d, _, _)| *d == civil)
        .map(|(_, title, rank)| (*title, *rank))
}
