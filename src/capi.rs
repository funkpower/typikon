//! A C ABI, so this can be embedded rather than spawned.
//!
//! Every desktop shell that consumes typikon today runs the binary and reads
//! its JSON, which costs about 1.5ms and is the right answer for a calendar
//! that changes once a day. This exists for the callers that cannot spawn, or
//! for whom a subprocess per redraw is the wrong shape: a LuaJIT statusline, a
//! C++ panel, anything with an FFI.
//!
//! # Shape
//!
//! Dates cross the boundary as **Julian Day Numbers**, so no struct layout is
//! part of the ABI and there is nothing to keep in step. `typikon_jdn_from_ymd`
//! converts.
//!
//! A `typikon_day` is opaque and owns its strings. Every accessor returns a
//! borrowed, NUL-terminated pointer valid until `typikon_day_free`, or NULL
//! where the day has no such field. The full JSON is available too, and it is
//! the same document the command line emits, under the same stability promise.
//!
//! # Errors
//!
//! A function that can fail returns NULL or 0 and leaves a message in
//! `typikon_last_error`, which is thread-local and valid until the next failing
//! call on the same thread.
//!
//! # Threads
//!
//! Every function is safe to call from any thread. A `typikon_day` is not
//! synchronised: do not use one from two threads at once.

use std::cell::RefCell;
use std::ffi::{c_char, c_int, CStr, CString};
use std::ptr;
use std::sync::OnceLock;

use crate::calendars::Date;
use crate::day::{self, Day};
use crate::fasting::Obligation;
use crate::jurisdictions;

/// Bumped when the meaning or signature of anything here changes. Additions
/// leave it alone.
const ABI_VERSION: u32 = 1;

thread_local! {
    static LAST_ERROR: RefCell<Option<CString>> = const { RefCell::new(None) };
}

fn set_error(msg: impl Into<Vec<u8>>) {
    let cs = CString::new(msg).unwrap_or_else(|_| c"error".into());
    LAST_ERROR.with(|e| *e.borrow_mut() = Some(cs));
}

fn clear_error() {
    LAST_ERROR.with(|e| *e.borrow_mut() = None);
}

/// The message from the last failing call on this thread, or NULL if the last
/// call succeeded. Valid until the next failing call on this thread.
#[no_mangle]
pub extern "C" fn typikon_last_error() -> *const c_char {
    LAST_ERROR.with(|e| match &*e.borrow() {
        Some(s) => s.as_ptr(),
        None => ptr::null(),
    })
}

/// The crate version, as a dotted string. Static; never freed.
#[no_mangle]
pub extern "C" fn typikon_version() -> *const c_char {
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr() as *const c_char
}

/// The ABI version of this header's contract.
#[no_mangle]
pub extern "C" fn typikon_abi_version() -> u32 {
    ABI_VERSION
}

/// # Safety
/// `p` must be NULL or a valid NUL-terminated C string.
unsafe fn opt_str<'a>(p: *const c_char) -> Option<&'a str> {
    if p.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(p) }.to_str().ok()
}

// ───────────────────────────────── dates ──────────────────────────────────

/// The Julian Day Number for a civil (Gregorian) date, or 0 if there is no
/// such date or it falls outside the years this calendar answers for.
#[no_mangle]
pub extern "C" fn typikon_jdn_from_ymd(year: i32, month: u32, day: u32) -> i32 {
    match Date::from_ymd(year, month, day) {
        Some(d) => {
            clear_error();
            d.jdn()
        }
        None => {
            set_error(format!("{year:04}-{month:02}-{day:02} is not a date this calendar reaches"));
            0
        }
    }
}

/// The civil date for a Julian Day Number. Returns 0 on success and -1 if the
/// number is outside the supported years, in which case the out-params are
/// untouched. Any out-param may be NULL.
///
/// # Safety
/// `year`, `month` and `day` must each be NULL or point to writable storage.
#[no_mangle]
pub unsafe extern "C" fn typikon_ymd_from_jdn(
    jdn: i32,
    year: *mut i32,
    month: *mut u32,
    day: *mut u32,
) -> c_int {
    let Some(d) = Date::from_jdn(jdn) else {
        set_error(format!("julian day {jdn} is outside the supported years"));
        return -1;
    };
    clear_error();
    let (y, m, dd) = d.ymd();
    unsafe {
        if !year.is_null() {
            *year = y;
        }
        if !month.is_null() {
            *month = m;
        }
        if !day.is_null() {
            *day = dd;
        }
    }
    0
}

/// Today's Julian Day Number, in local time.
#[no_mangle]
pub extern "C" fn typikon_today() -> i32 {
    clear_error();
    crate::local_today().jdn()
}

// ───────────────────────────────── a day ──────────────────────────────────

/// One reckoned day. Opaque: its layout is not part of the ABI.
///
/// Named in C's style rather than Rust's, because the name is what appears in
/// the header and a caller writing C should not have to translate.
#[allow(non_camel_case_types)]
pub struct typikon_day {
    inner: Day,
    /// Every string the accessors hand out, built once when the day is
    /// reckoned. An earlier version allocated on each call and kept the
    /// allocation alive until free, so a statusline polling `title` in a loop
    /// grew without bound.
    json: CString,
    iso: CString,
    weekday: CString,
    jurisdiction: CString,
    calendar: CString,
    tradition: CString,
    church_date: CString,
    title: CString,
    fast: CString,
    fast_label: CString,
    emoji: CString,
    reason: CString,
    season: Option<CString>,
    cycle: Option<CString>,
    week: Option<CString>,
    feast: Option<CString>,
    rank: Option<CString>,
    sign: Option<CString>,
    obligation: Option<CString>,
    binds: Option<CString>,
    commemorations: Vec<CString>,
}

#[allow(clippy::too_many_arguments)]
fn build(
    jdn: i32,
    jurisdiction: Option<&str>,
    tradition: Option<&str>,
    discipline: c_int,
) -> Option<Box<typikon_day>> {
    let Some(date) = Date::from_jdn(jdn) else {
        set_error(format!("julian day {jdn} is outside the supported years"));
        return None;
    };

    // -1 asks for whatever the config remembers; 0 and 1 override it.
    let typikon = match discipline {
        d if d < 0 => None,
        0 => Some(false),
        _ => Some(true),
    };
    let jur = match jurisdictions::resolve_full(jurisdiction, tradition, typikon) {
        Ok(j) => j,
        Err(e) => {
            set_error(e);
            return None;
        }
    };

    clear_error();
    let inner = day::describe(date, jur);
    let cs = |s: &str| CString::new(s).unwrap_or_default();
    Some(Box::new(typikon_day {
        json: cs(&serde_json::to_string(&inner.to_json()).ok()?),
        iso: cs(&inner.date.iso()),
        weekday: cs(inner.weekday.name()),
        jurisdiction: cs(inner.jurisdiction),
        calendar: cs(inner.calendar),
        tradition: cs(inner.tradition),
        church_date: cs(&inner.church_date),
        title: cs(inner.title()),
        fast: cs(inner.fast.key()),
        fast_label: cs(inner.fast_label),
        emoji: cs(inner.emoji),
        reason: cs(&inner.reason),
        season: inner.season.map(|s| cs(s.name())),
        cycle: inner.cycle.as_deref().map(&cs),
        week: inner.week.as_deref().map(&cs),
        feast: inner.feast.as_deref().map(&cs),
        rank: inner.rank.map(|r| cs(r.key())),
        sign: inner.rank.map(|r| cs(r.sign())),
        obligation: inner.obligation.map(|o| {
            cs(match o {
                Obligation::Obligatory => "obligatory",
                Obligation::Customary => "customary",
            })
        }),
        binds: inner.binds.map(|b| cs(b.key())),
        commemorations: inner.commemorations.iter().map(|c| cs(c)).collect(),
        inner,
    }))
}

/// Reckon a day. `jurisdiction` may be NULL to use whatever
/// `typikon --set-default` remembered. Returns NULL on error; see
/// `typikon_last_error`. Free it with `typikon_day_free`.
///
/// # Safety
/// `jurisdiction` must be NULL or a valid NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn typikon_describe(
    jdn: i32,
    jurisdiction: *const c_char,
) -> *mut typikon_day {
    match build(jdn, unsafe { opt_str(jurisdiction) }, None, -1) {
        Some(d) => Box::into_raw(d),
        None => ptr::null_mut(),
    }
}

/// As `typikon_describe`, overriding the usage and the discipline.
///
/// `tradition` is "slavic", "greek", or NULL to take the jurisdiction's own.
/// `discipline` is 1 to show the day at the Typikon's full discipline, 0 to
/// show only what the church's particular law requires, and -1 to use whatever
/// was remembered.
///
/// # Safety
/// `jurisdiction` and `tradition` must each be NULL or a valid NUL-terminated
/// C string.
#[no_mangle]
pub unsafe extern "C" fn typikon_describe_full(
    jdn: i32,
    jurisdiction: *const c_char,
    tradition: *const c_char,
    discipline: c_int,
) -> *mut typikon_day {
    let (j, t) = unsafe { (opt_str(jurisdiction), opt_str(tradition)) };
    match build(jdn, j, t, discipline) {
        Some(d) => Box::into_raw(d),
        None => ptr::null_mut(),
    }
}

/// Free a day. NULL is accepted and ignored.
///
/// # Safety
/// `day` must be NULL or a pointer returned by `typikon_describe` or
/// `typikon_describe_full` and not already freed.
#[no_mangle]
pub unsafe extern "C" fn typikon_day_free(day: *mut typikon_day) {
    if !day.is_null() {
        drop(unsafe { Box::from_raw(day) });
    }
}

macro_rules! accessor {
    ($name:ident, $doc:literal, |$d:ident| $body:expr) => {
        #[doc = $doc]
        ///
        /// Borrowed and NUL-terminated; valid until `typikon_day_free`. The
        /// same pointer is returned every time, so calling this in a loop
        /// allocates nothing.
        ///
        /// # Safety
        /// `day` must be NULL or a live pointer from `typikon_describe`.
        #[no_mangle]
        pub unsafe extern "C" fn $name(day: *const typikon_day) -> *const c_char {
            let Some($d) = (unsafe { day.as_ref() }) else {
                return ptr::null();
            };
            let value: Option<&CString> = $body;
            value.map_or(ptr::null(), |v| v.as_ptr())
        }
    };
}

accessor!(typikon_day_json, "The whole day as JSON, the same document the command line emits.", |d| Some(&d.json));
accessor!(typikon_day_iso, "The civil date, as \"2026-04-12\".", |d| Some(&d.iso));
accessor!(typikon_day_weekday, "The day of the week, in English.", |d| Some(&d.weekday));
accessor!(typikon_day_jurisdiction, "The key of the church whose reckoning this is, as `typikon_describe` takes it.", |d| Some(&d.jurisdiction));
accessor!(typikon_day_calendar, "The calendar: old, new or gregorian.", |d| Some(&d.calendar));
accessor!(typikon_day_tradition, "The usage the day was read under: slavic or greek.", |d| Some(&d.tradition));
accessor!(typikon_day_church_date, "The ecclesiastical date, as \"25 December\".", |d| Some(&d.church_date));
accessor!(typikon_day_title, "The feast, or the day's principal commemoration.", |d| Some(&d.title));
accessor!(typikon_day_fast, "The fasting level: strict, xerophagy, wine_oil, fish, dairy or none.", |d| Some(&d.fast));
accessor!(typikon_day_fast_label, "The fasting level as this tradition words it.", |d| Some(&d.fast_label));
accessor!(typikon_day_emoji, "The glyph for the fasting level.", |d| Some(&d.emoji));
accessor!(typikon_day_reason, "Why the day fasts as it does.", |d| Some(&d.reason));
accessor!(typikon_day_season, "The season, or NULL in ordinary time.", |d| d.season.as_ref());
accessor!(typikon_day_cycle, "The place in the movable cycle, or NULL outside it.", |d| d.cycle.as_ref());
accessor!(typikon_day_week, "The count of weeks from Pentecost, or NULL outside it.", |d| d.week.as_ref());
accessor!(typikon_day_feast, "The feast, or NULL if the day has none.", |d| d.feast.as_ref());
accessor!(typikon_day_rank, "The rank of the feast, or NULL if the day has none.", |d| d.rank.as_ref());
accessor!(typikon_day_sign, "The mark the books print beside a day of this rank, or NULL.", |d| d.sign.as_ref());
accessor!(typikon_day_obligation, "obligatory, customary, or NULL where the church does not legislate the day.", |d| d.obligation.as_ref());
accessor!(typikon_day_binds, "The minimum a particular law requires, or NULL where nothing binds.", |d| d.binds.as_ref());

/// Whether the day's fast binds under the church's own particular law.
///
/// # Safety
/// `day` must be a live pointer from `typikon_describe`.
#[no_mangle]
pub unsafe extern "C" fn typikon_day_is_obligatory(day: *const typikon_day) -> c_int {
    match unsafe { day.as_ref() } {
        Some(d) => d.inner.is_obligatory() as c_int,
        None => 0,
    }
}

/// Days to Pascha, signed, counting to the Pascha of this day's own civil
/// year. Negative once it has passed.
///
/// # Safety
/// `day` must be a live pointer from `typikon_describe`.
#[no_mangle]
pub unsafe extern "C" fn typikon_day_days_to_pascha(day: *const typikon_day) -> i32 {
    match unsafe { day.as_ref() } {
        Some(d) => d.inner.days_to_pascha,
        None => 0,
    }
}

/// The Julian Day Number of Pascha in this day's civil year, or 0 if `day` is
/// NULL.
///
/// # Safety
/// `day` must be a live pointer from `typikon_describe`.
#[no_mangle]
pub unsafe extern "C" fn typikon_day_pascha(day: *const typikon_day) -> i32 {
    match unsafe { day.as_ref() } {
        Some(d) => d.inner.pascha.jdn(),
        None => 0,
    }
}

/// How many commemorations the day carries.
///
/// # Safety
/// `day` must be NULL or a live pointer from `typikon_describe`.
#[no_mangle]
pub unsafe extern "C" fn typikon_day_commemoration_count(day: *const typikon_day) -> u32 {
    match unsafe { day.as_ref() } {
        Some(d) => d.commemorations.len() as u32,
        None => 0,
    }
}

/// The `i`th commemoration, or NULL if there is no such one. Borrowed, and
/// valid until `typikon_day_free`.
///
/// # Safety
/// `day` must be NULL or a live pointer from `typikon_describe`.
#[no_mangle]
pub unsafe extern "C" fn typikon_day_commemoration(
    day: *const typikon_day,
    i: u32,
) -> *const c_char {
    match unsafe { day.as_ref() } {
        Some(d) => d
            .commemorations
            .get(i as usize)
            .map_or(ptr::null(), |c| c.as_ptr()),
        None => ptr::null(),
    }
}

/// How far the Julian reckoning currently lags the Gregorian one: 13 until 14
/// March 2100, 14 after it. Reported for every jurisdiction, including those
/// that do not use it.
///
/// # Safety
/// `day` must be NULL or a live pointer from `typikon_describe`.
#[no_mangle]
pub unsafe extern "C" fn typikon_day_julian_offset(day: *const typikon_day) -> i32 {
    match unsafe { day.as_ref() } {
        Some(d) => d.inner.julian_offset,
        None => 0,
    }
}

/// The Julian Day Number of Pascha in a given civil year, by a given
/// jurisdiction's paschalion. Returns 0 on error.
///
/// Tradition and discipline cannot move Pascha, so they are not arguments.
///
/// # Safety
/// `jurisdiction` must be NULL or a valid NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn typikon_pascha(year: i32, jurisdiction: *const c_char) -> i32 {
    let jur = match jurisdictions::resolve_full(unsafe { opt_str(jurisdiction) }, None, None) {
        Ok(j) => j,
        Err(e) => {
            set_error(e);
            return 0;
        }
    };
    let (movables, _) = day::year_of(year, jur);
    clear_error();
    movables.pascha.jdn()
}

// ───────────────────────────── jurisdictions ──────────────────────────────

/// Key, name and note for every jurisdiction, NUL-terminated once and kept.
/// The table is a compile-time constant, so this is built on first use and
/// never changes.
fn jurisdiction_table() -> &'static [(CString, CString, CString)] {
    static TABLE: OnceLock<Vec<(CString, CString, CString)>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let cs = |s: &str| CString::new(s).unwrap_or_default();
        jurisdictions::JURISDICTIONS
            .iter()
            .chain(jurisdictions::CALENDARS)
            .map(|j| (cs(j.key), cs(j.name), cs(j.note)))
            .collect()
    })
}

/// How many jurisdictions this build knows. The named churches come first,
/// then the three bare calendars.
#[no_mangle]
pub extern "C" fn typikon_jurisdiction_count() -> u32 {
    jurisdiction_table().len() as u32
}

/// The `i`th jurisdiction's key, the one `typikon_describe` takes, or NULL if
/// there is no such one. Static storage; never freed.
#[no_mangle]
pub extern "C" fn typikon_jurisdiction_key(i: u32) -> *const c_char {
    jurisdiction_table()
        .get(i as usize)
        .map_or(ptr::null(), |j| j.0.as_ptr())
}

/// The `i`th jurisdiction's name, as it would be printed. Static storage.
#[no_mangle]
pub extern "C" fn typikon_jurisdiction_name(i: u32) -> *const c_char {
    jurisdiction_table()
        .get(i as usize)
        .map_or(ptr::null(), |j| j.1.as_ptr())
}

/// What is worth knowing about the `i`th jurisdiction's reckoning, which is
/// often the empty string. Static storage.
#[no_mangle]
pub extern "C" fn typikon_jurisdiction_note(i: u32) -> *const c_char {
    jurisdiction_table()
        .get(i as usize)
        .map_or(ptr::null(), |j| j.2.as_ptr())
}
