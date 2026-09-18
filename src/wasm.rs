//! JavaScript and TypeScript, through WebAssembly.
//!
//! # Shape
//!
//! A day comes back as a **plain object**, not a handle: the same document the
//! command line emits under `--json`, with the same field names and the same
//! stability promise. Nothing has to be freed, `JSON.stringify` works, and it
//! spreads into React state without ceremony. A wasm class would have meant a
//! `.free()` that every caller eventually forgets.
//!
//! # The clock and the config file
//!
//! WebAssembly has neither. `today()` asks the host what day it is, in the
//! host's own time zone, which is what a calendar wants. There is no config
//! file to remember a default jurisdiction in, so the default here is the New
//! Calendar and a caller who wants another passes it. That is the one place
//! this binding differs from the others, and it differs because a browser has
//! nowhere to put the answer.
//!
//! # The menologion
//!
//! The common menologion is compiled in, so the module is self-contained and
//! needs no network. The supplementary sets the command line can fetch are not
//! included: a caller that wants one passes it to `setMenaion`.

use serde::Serialize;
use wasm_bindgen::prelude::*;

use crate::calendars::Date;
use crate::day::{self, Day};
use crate::jurisdictions::{self, Jurisdiction};

/// Hand-written because the generated `.d.ts` would otherwise say `any`: a
/// binding that types its whole return value as `any` is a binding that
/// TypeScript cannot check.
#[wasm_bindgen(typescript_custom_section)]
const TYPES: &'static str = r#"
/** One day, fully reckoned. The same document `typikon --json` emits. */
export interface Day {
  /** The civil date, as "2026-04-12". */
  date: string;
  weekday: string;
  /** The key of the church whose reckoning this is. */
  jurisdiction: string;
  /** "old", "new" or "western". */
  calendar: string;
  /** The ecclesiastical date, as "25 December". */
  churchDate: string;
  /** null in ordinary time. */
  season: string | null;
  /** The place in the movable cycle, or null outside it. */
  cycle: string | null;
  feast: string | null;
  /** The rank of the feast, or null where there is none. */
  rank: string | null;
  /** The mark the books print beside a day of this rank. */
  sign: string | null;
  /** "slavic" or "greek". */
  tradition: string;
  /** The feast, or the day's principal commemoration. */
  title: string;
  commemorations: string[];
  /** "strict", "xerophagy", "wine_oil", "fish", "dairy" or "none". */
  fast: string;
  /** The level as this tradition words it. */
  fastLabel: string;
  /** The glyph for the level. */
  emoji: string;
  /** Why the day fasts as it does. */
  reason: string;
  /** "obligatory", "customary", or null where the church does not legislate. */
  obligation: string | null;
  /** The minimum a particular law requires, or null where nothing binds. */
  binds: string | null;
  /** The civil date of Pascha in this day's own year. */
  pascha: string;
  /** Signed, to the Pascha of this day's own civil year. */
  daysToPascha: number;
  /** How far the Julian reckoning lags the Gregorian: 13 until 14 March 2100. */
  julianOffset: number;
}

/** Whose reckoning to keep, and how much of it to show. */
export interface Options {
  /** A jurisdiction key, or a bare calendar: "old", "new", "western". */
  jurisdiction?: string;
  /** "slavic" or "greek", overriding the jurisdiction's own usage. */
  tradition?: string;
  /**
   * True to show the day at the Typikon's full discipline rather than at the
   * minimum the church's own particular law requires. What binds is reported
   * either way. This is for the Byzantine Catholic churches, whose law asks
   * very much less than the tradition they keep.
   */
  typikon?: boolean;
}

export interface JurisdictionInfo {
  key: string;
  name: string;
  note: string;
}
"#;

/// The options object, as it arrives.
#[derive(Default, serde::Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct Options {
    jurisdiction: Option<String>,
    tradition: Option<String>,
    typikon: Option<bool>,
}

impl Options {
    /// The named jurisdiction, or the New Calendar, with the usage and the
    /// discipline applied.
    fn resolve(&self) -> Result<&'static Jurisdiction, JsError> {
        jurisdictions::resolve_full(
            Some(self.jurisdiction.as_deref().unwrap_or("new")),
            self.tradition.as_deref(),
            self.typikon,
        )
        .map_err(|e| JsError::new(&e))
    }
}

fn parse_options(value: Option<js_sys::Object>) -> Result<Options, JsError> {
    let Some(value) = value else { return Ok(Options::default()) };
    serde_wasm_bindgen::from_value(value.into()).map_err(|e| JsError::new(&e.to_string()))
}

/// Serialize with plain JS objects and arrays rather than Maps, and an absent
/// field as `null` rather than `undefined`.
///
/// Both defaults would otherwise break the promise that this is the document
/// `--json` emits: `JSON.stringify` drops a key whose value is `undefined`, so
/// a day with no feast would round-trip without a `feast` key at all, and the
/// declared type says `string | null`.
fn to_js<T: Serialize>(value: &T) -> Result<JsValue, JsError> {
    let ser = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(true)
        .serialize_missing_as_null(true);
    value
        .serialize(&ser)
        .map_err(|e| JsError::new(&e.to_string()))
}

fn day_to_js(d: &Day) -> Result<JsValue, JsError> {
    to_js(&d.to_json())
}

/// A JS `Date`, read in the host's own time zone.
///
/// The liturgical day turns at local midnight, so reading UTC would report
/// yesterday's fast all evening for anyone west of Greenwich.
fn from_js_date(d: &js_sys::Date) -> Result<Date, JsError> {
    Date::from_ymd(
        d.get_full_year() as i32,
        d.get_month() + 1, // JS months are zero-based
        d.get_date(),
    )
    .ok_or_else(|| JsError::new("that is not a date this calendar reaches"))
}

fn to_js_date(d: Date) -> js_sys::Date {
    let (y, m, day) = d.ymd();
    js_sys::Date::new_with_year_month_day(y as u32, m as i32 - 1, day as i32)
}

// ──────────────────────────────── the api ─────────────────────────────────

/// One day, fully reckoned.
#[wasm_bindgen(unchecked_return_type = "Day")]
pub fn describe(
    date: &js_sys::Date,
    #[wasm_bindgen(unchecked_param_type = "Options")] options: Option<js_sys::Object>,
) -> Result<JsValue, JsError> {
    let opts = parse_options(options)?;
    day_to_js(&day::describe(from_js_date(date)?, opts.resolve()?))
}

/// One day, from an ISO date, as "2026-04-12".
#[wasm_bindgen(js_name = describeIso, unchecked_return_type = "Day")]
pub fn describe_iso(
    date: &str,
    #[wasm_bindgen(unchecked_param_type = "Options")] options: Option<js_sys::Object>,
) -> Result<JsValue, JsError> {
    let opts = parse_options(options)?;
    let d = Date::parse_iso(date)
        .ok_or_else(|| JsError::new(&format!("{date:?} is not a date this calendar reaches")))?;
    day_to_js(&day::describe(d, opts.resolve()?))
}

/// Today, by the host's clock and in the host's time zone.
#[wasm_bindgen(unchecked_return_type = "Day")]
pub fn today(
    #[wasm_bindgen(unchecked_param_type = "Options")] options: Option<js_sys::Object>,
) -> Result<JsValue, JsError> {
    describe(&js_sys::Date::new_0(), options)
}

/// `days` consecutive days from `start`, which is what a week view wants and
/// what it would otherwise get by calling `describe` seven times and paying
/// for the year's movable cycle each time.
#[wasm_bindgen(unchecked_return_type = "Day[]")]
pub fn week(
    start: &js_sys::Date,
    days: i32,
    #[wasm_bindgen(unchecked_param_type = "Options")] options: Option<js_sys::Object>,
) -> Result<JsValue, JsError> {
    if !(1..=36600).contains(&days) {
        return Err(JsError::new("days must be between 1 and 36600"));
    }
    let opts = parse_options(options)?;
    let out = day::week(from_js_date(start)?, days, opts.resolve()?);
    let arr = js_sys::Array::new();
    for d in &out {
        arr.push(&day_to_js(d)?);
    }
    Ok(arr.into())
}

/// Pascha in a civil year, by a jurisdiction's paschalion. Tradition and
/// discipline cannot move it, so they are not arguments.
#[wasm_bindgen]
pub fn pascha(year: i32, jurisdiction: Option<String>) -> Result<js_sys::Date, JsError> {
    let jur = jurisdictions::resolve_full(
        Some(jurisdiction.as_deref().unwrap_or("new")),
        None,
        None,
    )
    .map_err(|e| JsError::new(&e))?;
    let (movables, _) = day::year_of(year, jur);
    Ok(to_js_date(movables.pascha))
}

/// Every jurisdiction this build knows: the named churches first, then the
/// three bare calendars.
#[wasm_bindgen(unchecked_return_type = "JurisdictionInfo[]")]
pub fn jurisdictions() -> Result<JsValue, JsError> {
    #[derive(Serialize)]
    struct Info<'a> {
        key: &'a str,
        name: &'a str,
        note: &'a str,
    }
    let out: Vec<Info> = jurisdictions::JURISDICTIONS
        .iter()
        .chain(jurisdictions::CALENDARS)
        .map(|j| Info { key: j.key, name: j.name, note: j.note })
        .collect();
    to_js(&out)
}

/// The version of the crate this module was built from.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
