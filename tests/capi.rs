//! The C ABI, exercised as a C caller would use it.
//!
//! These run only under `--features capi`, which is what builds the module.
//! They cover three things a Rust test can check and a C compiler cannot: that
//! the answers match what the crate gives natively, that the pointers an
//! accessor hands out stay put, and that the header in `include/` still
//! describes the library that is actually built.

#![cfg(feature = "capi")]

use std::ffi::{c_char, CStr, CString};
use std::ptr;

use typikon::capi::*;

/// Borrow a returned pointer as a `str`, or `None` for NULL.
///
/// # Safety
/// Only called on pointers these functions returned, which live until free.
fn s<'a>(p: *const c_char) -> Option<&'a str> {
    if p.is_null() {
        None
    } else {
        Some(unsafe { CStr::from_ptr(p) }.to_str().expect("utf-8"))
    }
}

fn describe(iso: &str, jurisdiction: &str) -> *mut typikon_day {
    let (y, m, d) = (
        iso[0..4].parse().unwrap(),
        iso[5..7].parse().unwrap(),
        iso[8..10].parse().unwrap(),
    );
    let jdn = typikon_jdn_from_ymd(y, m, d);
    assert_ne!(jdn, 0, "{iso} should be a date");
    let key = CString::new(jurisdiction).unwrap();
    let day = unsafe { typikon_describe(jdn, key.as_ptr()) };
    assert!(!day.is_null(), "{iso} {jurisdiction} should reckon");
    day
}

// ──────────────────────────────── dates ───────────────────────────────────

#[test]
fn jdn_round_trips() {
    for (y, m, d) in [(1, 1, 1), (1583, 10, 15), (2026, 4, 12), (9999, 12, 31)] {
        let jdn = typikon_jdn_from_ymd(y, m, d);
        assert_ne!(jdn, 0, "{y}-{m}-{d}");
        let (mut yy, mut mm, mut dd) = (0i32, 0u32, 0u32);
        let rc = unsafe { typikon_ymd_from_jdn(jdn, &mut yy, &mut mm, &mut dd) };
        assert_eq!(rc, 0);
        assert_eq!((yy, mm, dd), (y, m, d));
    }
}

#[test]
fn an_impossible_date_is_an_error_not_a_guess() {
    assert_eq!(typikon_jdn_from_ymd(2026, 2, 30), 0);
    let msg = s(typikon_last_error()).expect("a message");
    assert!(msg.contains("2026-02-30"), "{msg}");

    // And the next success clears it, so a caller that only checks the error
    // pointer cannot read a stale one.
    assert_ne!(typikon_jdn_from_ymd(2026, 2, 28), 0);
    assert!(typikon_last_error().is_null());
}

#[test]
fn ymd_out_params_are_each_optional() {
    let jdn = typikon_jdn_from_ymd(2026, 4, 12);
    let mut year = 0i32;
    let rc = unsafe { typikon_ymd_from_jdn(jdn, &mut year, ptr::null_mut(), ptr::null_mut()) };
    assert_eq!(rc, 0);
    assert_eq!(year, 2026);
}

#[test]
fn a_julian_day_outside_the_years_is_refused() {
    let mut year = 12345i32;
    let rc = unsafe { typikon_ymd_from_jdn(-1, &mut year, ptr::null_mut(), ptr::null_mut()) };
    assert_eq!(rc, -1);
    assert_eq!(year, 12345, "out-params are untouched on failure");
    assert!(s(typikon_last_error()).is_some());
}

// ───────────────────────────────── a day ──────────────────────────────────

#[test]
fn a_day_answers_what_the_crate_answers() {
    let jur = typikon::jurisdictions::resolve(Some("moscow")).unwrap();
    let native = typikon::day::describe(
        typikon::calendars::Date::parse_iso("2026-04-12").unwrap(),
        jur,
    );

    let day = describe("2026-04-12", "moscow");
    unsafe {
        assert_eq!(s(typikon_day_iso(day)), Some("2026-04-12"));
        assert_eq!(s(typikon_day_title(day)), Some(native.title()));
        assert_eq!(s(typikon_day_fast(day)), Some(native.fast.key()));
        assert_eq!(s(typikon_day_emoji(day)), Some(native.emoji));
        assert_eq!(s(typikon_day_jurisdiction(day)), Some(native.jurisdiction));
        assert_eq!(s(typikon_day_church_date(day)), Some(&*native.church_date));
        assert_eq!(typikon_day_days_to_pascha(day), native.days_to_pascha);
        assert_eq!(typikon_day_pascha(day), native.pascha.jdn());
        assert_eq!(typikon_day_julian_offset(day), native.julian_offset);
        assert_eq!(
            typikon_day_commemoration_count(day) as usize,
            native.commemorations.len()
        );
        typikon_day_free(day);
    }
}

#[test]
fn the_json_is_the_document_the_command_line_emits() {
    let day = describe("2026-01-07", "moscow");
    let json = s(unsafe { typikon_day_json(day) }).expect("json");
    let parsed: serde_json::Value = serde_json::from_str(json).expect("valid json");

    let jur = typikon::jurisdictions::resolve(Some("moscow")).unwrap();
    let native = typikon::day::describe(
        typikon::calendars::Date::parse_iso("2026-01-07").unwrap(),
        jur,
    );
    let expected = serde_json::to_value(native.to_json()).unwrap();
    assert_eq!(parsed, expected);

    unsafe { typikon_day_free(day) };
}

#[test]
fn an_accessor_returns_the_same_pointer_every_time() {
    // The point of building the strings once: a statusline that reads `title`
    // on every redraw must not allocate on every redraw. An earlier version
    // did, and kept every allocation alive until free.
    let day = describe("2026-04-12", "moscow");
    unsafe {
        let first = typikon_day_title(day);
        for _ in 0..10_000 {
            assert_eq!(typikon_day_title(day), first);
        }
        typikon_day_free(day);
    }
}

#[test]
fn a_null_day_answers_null_rather_than_crashing() {
    let n: *const typikon_day = ptr::null();
    unsafe {
        assert!(typikon_day_json(n).is_null());
        assert!(typikon_day_title(n).is_null());
        assert!(typikon_day_season(n).is_null());
        assert!(typikon_day_commemoration(n, 0).is_null());
        assert_eq!(typikon_day_commemoration_count(n), 0);
        assert_eq!(typikon_day_is_obligatory(n), 0);
        assert_eq!(typikon_day_days_to_pascha(n), 0);
        assert_eq!(typikon_day_pascha(n), 0);
        assert_eq!(typikon_day_julian_offset(n), 0);
        typikon_day_free(ptr::null_mut()); // accepted and ignored
    }
}

#[test]
fn a_commemoration_past_the_end_is_null() {
    let day = describe("2026-04-12", "moscow");
    unsafe {
        let n = typikon_day_commemoration_count(day);
        assert!(typikon_day_commemoration(day, n).is_null());
        assert!(typikon_day_commemoration(day, u32::MAX).is_null());
        if n > 0 {
            assert!(!typikon_day_commemoration(day, 0).is_null());
        }
        typikon_day_free(day);
    }
}

#[test]
fn a_jurisdiction_nobody_keeps_is_an_error() {
    let key = CString::new("shire").unwrap();
    let day = unsafe { typikon_describe(typikon_today(), key.as_ptr()) };
    assert!(day.is_null());
    let msg = s(typikon_last_error()).expect("a message");
    assert!(msg.contains("shire"), "{msg}");
}

#[test]
fn a_null_jurisdiction_takes_the_remembered_default() {
    let day = unsafe { typikon_describe(typikon_jdn_from_ymd(2026, 4, 12), ptr::null()) };
    assert!(!day.is_null());
    unsafe {
        assert!(s(typikon_day_jurisdiction(day)).is_some());
        typikon_day_free(day);
    }
}

#[test]
fn the_discipline_flag_moves_what_is_shown_but_not_what_binds() {
    // The Byzantine Catholic case the flag exists for: on a Wednesday of the
    // Great Fast, Ruthenian law requires nothing, while the Typikon it inherits
    // keeps xerophagy.
    let jdn = typikon_jdn_from_ymd(2026, 3, 4); // Wednesday, Great Fast
    let key = CString::new("ruthenian").unwrap();
    unsafe {
        let law = typikon_describe_full(jdn, key.as_ptr(), ptr::null(), 0);
        let typikon = typikon_describe_full(jdn, key.as_ptr(), ptr::null(), 1);
        assert!(!law.is_null() && !typikon.is_null());

        assert_ne!(
            s(typikon_day_fast(law)),
            s(typikon_day_fast(typikon)),
            "the Typikon should keep more than the law requires"
        );
        assert_eq!(
            s(typikon_day_binds(law)),
            s(typikon_day_binds(typikon)),
            "what binds is the law either way"
        );
        typikon_day_free(law);
        typikon_day_free(typikon);
    }
}

#[test]
fn the_usage_can_be_overridden() {
    // Fish in the Nativity Fast: Greek practice permits it on a weekday before
    // 17 December, Slavic practice keeps it to Saturdays and Sundays.
    let jdn = typikon_jdn_from_ymd(2025, 12, 9); // a Tuesday
    let key = CString::new("old").unwrap();
    let (greek, slavic) = (CString::new("greek").unwrap(), CString::new("slavic").unwrap());
    unsafe {
        let g = typikon_describe_full(jdn, key.as_ptr(), greek.as_ptr(), -1);
        let sl = typikon_describe_full(jdn, key.as_ptr(), slavic.as_ptr(), -1);
        assert_eq!(s(typikon_day_tradition(g)), Some("greek"));
        assert_eq!(s(typikon_day_tradition(sl)), Some("slavic"));
        typikon_day_free(g);
        typikon_day_free(sl);
    }
}

#[test]
fn an_unknown_usage_is_an_error() {
    let key = CString::new("moscow").unwrap();
    let bad = CString::new("frankish").unwrap();
    let day = unsafe { typikon_describe_full(typikon_today(), key.as_ptr(), bad.as_ptr(), -1) };
    assert!(day.is_null());
    assert!(s(typikon_last_error()).is_some());
}

// ────────────────────────── without a day ─────────────────────────────────

#[test]
fn pascha_by_year_matches_the_published_dates() {
    let old = CString::new("old").unwrap();
    let west = CString::new("western").unwrap();
    for (year, julian, gregorian) in [
        (2026, "2026-04-12", "2026-04-05"),
        (2027, "2027-05-02", "2027-03-28"),
        (2028, "2028-04-16", "2028-04-16"),
    ] {
        for (key, expected) in [(&old, julian), (&west, gregorian)] {
            let jdn = unsafe { typikon_pascha(year, key.as_ptr()) };
            let (mut y, mut m, mut d) = (0i32, 0u32, 0u32);
            assert_eq!(unsafe { typikon_ymd_from_jdn(jdn, &mut y, &mut m, &mut d) }, 0);
            assert_eq!(format!("{y:04}-{m:02}-{d:02}"), expected);
        }
    }
}

#[test]
fn pascha_refuses_a_jurisdiction_nobody_keeps() {
    let key = CString::new("shire").unwrap();
    assert_eq!(unsafe { typikon_pascha(2026, key.as_ptr()) }, 0);
    assert!(s(typikon_last_error()).is_some());
}

#[test]
fn every_jurisdiction_is_listed_and_every_listed_key_reckons() {
    let n = typikon_jurisdiction_count();
    assert_eq!(
        n as usize,
        typikon::jurisdictions::JURISDICTIONS.len() + typikon::jurisdictions::CALENDARS.len()
    );
    for i in 0..n {
        let key = s(typikon_jurisdiction_key(i)).expect("a key");
        assert!(s(typikon_jurisdiction_name(i)).is_some_and(|x| !x.is_empty()));
        assert!(s(typikon_jurisdiction_note(i)).is_some(), "note may be empty, not NULL");
        let day = describe("2026-04-12", key);
        unsafe { typikon_day_free(day) };
    }
    assert!(typikon_jurisdiction_key(n).is_null());
}

#[test]
fn the_version_is_the_crate_version() {
    assert_eq!(s(typikon_version()), Some(env!("CARGO_PKG_VERSION")));
}

// ──────────────────────────────── the header ──────────────────────────────

/// Every symbol the library exports is declared in `include/typikon.h`, and
/// every function the header declares is exported.
///
/// A header is a promise made in a second file, and a second file drifts. This
/// is the cheapest check that it has not.
#[test]
fn the_header_describes_the_library() {
    let src = std::fs::read_to_string("src/capi.rs").expect("src/capi.rs");
    let header = std::fs::read_to_string("include/typikon.h").expect("include/typikon.h");

    let mut exported: Vec<&str> = Vec::new();
    for line in src.lines() {
        // `fn typikon_foo(` in a line that declares an extern function, plus
        // the names the accessor macro generates, which are its first argument.
        if let Some(rest) = line.trim().strip_prefix("pub extern \"C\" fn ") {
            exported.push(rest.split('(').next().unwrap());
        } else if let Some(rest) = line.trim().strip_prefix("pub unsafe extern \"C\" fn ") {
            exported.push(rest.split('(').next().unwrap());
        } else if let Some(rest) = line.trim().strip_prefix("accessor!(") {
            exported.push(rest.split(',').next().unwrap());
        }
    }
    // The macro's own definition line reads as `fn $name(`; the names it
    // generates are picked up from the invocations.
    exported.retain(|n| !n.starts_with('$'));
    assert!(exported.len() > 20, "found only {} symbols", exported.len());

    for name in &exported {
        assert!(
            header.contains(&format!("{name}(")),
            "{name} is exported but not declared in include/typikon.h"
        );
    }

    for line in header.lines() {
        let Some(start) = line.find("typikon_") else { continue };
        if !line.contains('(') || line.trim_start().starts_with('*') {
            continue;
        }
        let name: String = line[start..]
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if !line[start + name.len()..].starts_with('(') {
            continue; // a mention in prose, not a declaration
        }
        assert!(
            exported.contains(&name.as_str()),
            "{name} is declared in include/typikon.h but not exported"
        );
    }
}

/// The header hard-codes the ABI version so a caller can compare what it
/// compiled against with what it loaded.
#[test]
fn the_headers_abi_version_matches_the_library() {
    let header = std::fs::read_to_string("include/typikon.h").expect("include/typikon.h");
    let line = header
        .lines()
        .find(|l| l.starts_with("#define TYPIKON_ABI_VERSION"))
        .expect("the header defines it");
    let declared: u32 = line
        .split_whitespace()
        .next_back()
        .unwrap()
        .trim_end_matches('u')
        .parse()
        .expect("a number");
    assert_eq!(declared, typikon_abi_version());
}

/// The LuaJIT binding declares the library it loads, and a declaration that
/// has drifted from the header is a segfault rather than a compile error.
#[test]
fn the_lua_binding_declares_the_whole_header() {
    let header = std::fs::read_to_string("include/typikon.h").expect("include/typikon.h");
    let lua = std::fs::read_to_string("examples/lua/typikon.lua").expect("the lua binding");
    let cdef = {
        let start = lua.find("ffi.cdef").expect("a cdef block");
        let body = &lua[start..];
        let end = body.find("]]").expect("the block ends");
        &body[..end]
    };

    for line in header.lines() {
        let Some(start) = line.find("typikon_") else { continue };
        let name: String = line[start..]
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if !line[start + name.len()..].starts_with('(') {
            continue;
        }
        assert!(
            cdef.contains(&format!("{name}(")),
            "{name} is in the header but not in the lua cdef"
        );
    }
}
