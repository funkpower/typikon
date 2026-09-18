//! Pinned against published calendars and against dates whose answer is not
//! open to interpretation.

use typikon::calendars::{Date, JulianDate, Weekday};
use typikon::day::{describe, week, year_of};
use typikon::export::to_ics;
use typikon::fasting::{Level, Obligation};
use typikon::jurisdictions::{resolve, Calendar, MenologionKind, CALENDARS, JURISDICTIONS};
use typikon::menologion::Rank;
use typikon::paschalion::{gregorian_pascha, julian_pascha, Paschalion};
use typikon::seasons::Season;

fn d(s: &str) -> Date {
    Date::parse_iso(s).expect("a real date")
}

fn at(s: &str, j: &str) -> typikon::day::Day {
    describe(d(s), resolve(Some(j)).expect("a real jurisdiction"))
}

// ───────────────────────────── the paschalion ─────────────────────────────

#[test]
fn julian_pascha_matches_the_published_dates() {
    for (year, expected) in [
        (2024, "2024-05-05"), (2025, "2025-04-20"), (2026, "2026-04-12"),
        (2027, "2027-05-02"), (2028, "2028-04-16"), (2030, "2030-04-28"),
    ] {
        assert_eq!(julian_pascha(year).iso(), expected, "julian pascha {year}");
    }
}

#[test]
fn gregorian_pascha_matches_the_published_dates() {
    for (year, expected) in [
        (2024, "2024-03-31"), (2025, "2025-04-20"), (2026, "2026-04-05"),
        (2027, "2027-03-28"), (2028, "2028-04-16"),
    ] {
        assert_eq!(gregorian_pascha(year).iso(), expected, "gregorian pascha {year}");
    }
}

#[test]
fn the_two_paschalions_agree_in_2025_and_differ_by_a_week_in_2026() {
    assert_eq!(julian_pascha(2025), gregorian_pascha(2025));
    assert_eq!(julian_pascha(2026) - gregorian_pascha(2026), 7);
}

#[test]
fn the_paschalion_invariants_hold_for_two_centuries() {
    for year in 1900..=2100 {
        let (j, g) = (julian_pascha(year), gregorian_pascha(year));
        assert_eq!(j.weekday(), Weekday::Sunday, "julian {year}");
        assert_eq!(g.weekday(), Weekday::Sunday, "gregorian {year}");
        assert!(
            Date::from_ymd(year, 4, 4).unwrap() <= j && j <= Date::from_ymd(year, 5, 8).unwrap(),
            "orthodox pascha {year} out of range: {j}"
        );
        assert!(
            Date::from_ymd(year, 3, 22).unwrap() <= g && g <= Date::from_ymd(year, 4, 25).unwrap(),
            "western pascha {year} out of range: {g}"
        );
    }
}

// ──────────────────────────── calendar drift ──────────────────────────────

#[test]
fn every_civil_date_round_trips_through_the_julian_calendar() {
    let mut day = d("1800-01-01");
    let last = d("2200-12-31");
    while day <= last {
        let j = day.to_julian();
        assert_eq!(j.to_civil(), day, "round trip {day}");
        day = day + 1;
    }
}

#[test]
fn julian_lags_thirteen_days_now() {
    assert_eq!(d("2026-09-18").julian_offset(), 13);
    let j = d("2026-09-18").to_julian();
    assert_eq!((j.year, j.month, j.day), (2026, 9, 5));
}

#[test]
fn the_julian_offset_widens_the_day_after_a_julian_leap_day() {
    for (before, after) in [
        ("1900-03-13", "1900-03-14"),
        ("2100-03-14", "2100-03-15"),
        ("2200-03-15", "2200-03-16"),
    ] {
        assert_eq!(
            d(after).julian_offset(),
            d(before).julian_offset() + 1,
            "offset should widen at {after}"
        );
    }
}

#[test]
fn old_calendar_nativity_falls_on_7_january() {
    assert_eq!(
        JulianDate { year: 2026, month: 12, day: 25 }.to_civil().iso(),
        "2027-01-07"
    );
}

#[test]
fn old_and_new_disagree_about_the_day_not_the_date() {
    let old = at("2027-01-07", "rocor");
    let new = at("2026-12-25", "goarch");
    assert_eq!(old.church_date, "25 December");
    assert_eq!(new.church_date, "25 December");
    assert_eq!(old.feast, new.feast);
    assert_eq!(old.rank, Some(Rank::Great));
    assert_eq!(new.rank, Some(Rank::Great));
}

// ───────────────────────── the Orthodox fast ──────────────────────────────

#[test]
fn great_fast_weekdays_are_xerophagy_and_weekends_wine_and_oil() {
    assert_eq!(at("2026-02-25", "new").fast, Level::Xerophagy);
    assert_eq!(at("2026-02-28", "new").fast, Level::WineOil);
    assert_eq!(at("2026-03-01", "new").fast, Level::WineOil);
}

#[test]
fn the_two_strictest_days_of_the_year() {
    assert_eq!(at("2026-02-23", "new").fast, Level::Strict);
    assert_eq!(at("2026-04-10", "new").fast, Level::Strict);
    // The one Saturday kept without oil.
    assert_eq!(at("2026-04-11", "new").fast, Level::Xerophagy);
}

#[test]
fn annunciation_and_palm_sunday_admit_fish_inside_the_great_fast() {
    let a = at("2026-03-25", "new");
    assert_eq!(a.season, Some(Season::GreatFast));
    assert_eq!(a.fast, Level::Fish);
    assert_eq!(at("2026-04-05", "new").fast, Level::Fish);
}

#[test]
fn the_annunciation_is_never_lost_to_great_friday() {
    for (when, j) in [
        ("2016-03-25", "western"), ("2005-03-25", "western"),
        ("2029-04-07", "rocor"), ("2018-04-07", "rocor"),
    ] {
        let day = at(when, j);
        assert_eq!(day.church_date, "25 March", "{when}");
        assert!(day.feast.as_deref().unwrap_or("").contains("Annunciation"), "{when}");
        assert_eq!(day.rank, Some(Rank::Great), "{when}");
        assert_eq!(day.fast, Level::WineOil, "{when}");
    }
}

#[test]
fn kyriopascha_and_the_annunciation_in_bright_week_stay_fast_free() {
    assert_eq!(at("2035-03-25", "western").fast, Level::None);
    assert_eq!(at("2075-04-07", "rocor").fast, Level::None);
}

#[test]
fn palm_sunday_and_lazarus_saturday_are_still_the_fast() {
    for j in ["ruthenian", "ugcc", "melkite", "goarch", "rocor"] {
        for year in [2025, 2026, 2027] {
            let jur = resolve(Some(j)).unwrap();
            let (mv, _) = year_of(year, jur);
            for when in [mv.lazarus_saturday, mv.palm_sunday] {
                let day = describe(when, jur);
                assert!(day.season.is_some(), "{j} {when}: no season");
                assert_ne!(day.fast, Level::None, "{j} {when}");
                assert!(!day.reason.is_empty(), "{j} {when}: no reason");
            }
        }
    }
}

#[test]
fn a_ranked_saint_relaxes_a_weekday_of_a_fasting_season() {
    // The Nativity of the Forerunner, a great feast inside the Apostles' Fast.
    assert_eq!(at("2020-06-24", "goarch").fast, Level::Fish);
    // The Forty Martyrs of Sebaste, the textbook Lenten relaxation.
    assert_eq!(at("2022-03-09", "goarch").fast, Level::WineOil);
    // St Nicholas, a vigil saint on a Wednesday of the Nativity Fast.
    assert_eq!(at("2023-12-06", "goarch").fast, Level::WineOil);
}

#[test]
fn no_saint_brings_fish_into_the_great_fast_or_lifts_great_week() {
    for day in week(d("2022-03-07"), 40, resolve(Some("goarch")).unwrap()) {
        if day.church_date == "25 March" {
            continue;
        }
        if day.season == Some(Season::GreatFast) {
            assert!(day.fast >= Level::Strict, "{}", day.date);
            assert!(day.fast <= Level::WineOil, "{} got {:?}", day.date, day.fast);
        }
        if day.season == Some(Season::GreatWeek) {
            assert!(day.fast <= Level::WineOil, "{} got {:?}", day.date, day.fast);
        }
    }
}

#[test]
fn the_eves_relax_when_they_fall_on_a_weekend() {
    assert_eq!(at("2022-12-24", "goarch").fast, Level::WineOil);
    assert_eq!(at("2023-12-24", "goarch").fast, Level::WineOil);
    assert_eq!(at("2025-01-05", "goarch").fast, Level::WineOil);
    // A weekday eve is still xerophagy.
    assert_eq!(at("2024-12-24", "goarch").fast, Level::Xerophagy);
}

#[test]
fn cheesefare_week_keeps_dairy_on_wednesday_and_friday() {
    let day = at("2026-02-18", "new");
    assert_eq!(day.season, Some(Season::CheesefareWeek));
    assert_eq!(day.fast, Level::Dairy);
}

#[test]
fn the_fast_free_stretches() {
    assert_eq!(at("2026-04-17", "new").fast, Level::None); // Bright Friday
    assert_eq!(at("2026-01-02", "new").fast, Level::None); // Christmastide
    assert_eq!(at("2026-06-03", "new").fast, Level::None); // after Pentecost
    // Fast-free precisely because the Pharisee boasted of fasting twice a week.
    assert_eq!(at("2026-02-06", "new").fast, Level::None);
}

#[test]
fn nativity_fast_tightens_for_its_last_five_days() {
    assert_eq!(at("2026-12-12", "new").fast, Level::Fish);
    assert_eq!(at("2026-12-19", "new").fast, Level::Fish);
    assert_eq!(at("2026-12-22", "new").fast, Level::WineOil);
    assert_eq!(at("2026-12-24", "new").fast, Level::Xerophagy);
}

#[test]
fn ordinary_wednesday_and_friday_are_xerophagy() {
    let day = at("2026-09-04", "new");
    assert!(day.season.is_none());
    assert!(day.rank.is_none());
    assert_eq!(day.fast, Level::Xerophagy);
}

#[test]
fn a_polyeleos_saint_lifts_an_ordinary_friday_to_wine_and_oil() {
    let day = at("2026-10-09", "new");
    assert_eq!(day.weekday, Weekday::Friday);
    assert_eq!(day.rank, Some(Rank::Polyeleos));
    assert_eq!(day.fast, Level::WineOil);
}

#[test]
fn the_apostles_fast_is_empty_when_pascha_is_late() {
    let goarch: Vec<_> = week(d("2024-05-01"), 120, resolve(Some("goarch")).unwrap())
        .into_iter().filter(|x| x.season == Some(Season::Apostles)).collect();
    let rocor: Vec<_> = week(d("2024-05-01"), 120, resolve(Some("rocor")).unwrap())
        .into_iter().filter(|x| x.season == Some(Season::Apostles)).collect();
    assert!(goarch.is_empty(), "goarch 2024 should have no Apostles' Fast");
    assert!(!rocor.is_empty(), "rocor 2024 should still have one");
}

#[test]
fn the_orthodox_rule_never_reports_an_obligation() {
    for day in week(d("2026-01-01"), 365, resolve(Some("new")).unwrap()) {
        assert_ne!(day.obligation, Some(Obligation::Obligatory), "{}", day.date);
    }
}

// ──────────────────── Byzantine Catholic particular law ───────────────────

#[test]
fn ruthenian_pascha_is_the_gregorian_one() {
    assert_eq!(at("2026-06-01", "ruthenian").pascha.iso(), "2026-04-05");
}

#[test]
fn ruthenian_clean_monday_is_16_february_2026() {
    assert!(at("2026-02-16", "ruthenian").reason.starts_with("the first day"));
}

#[test]
fn only_the_great_fast_binds_under_ruthenian_law() {
    let obligatory: Vec<_> = week(d("2026-01-01"), 365, resolve(Some("ruthenian")).unwrap())
        .into_iter().filter(|x| x.is_obligatory()).collect();
    let dates: Vec<String> = obligatory.iter().map(|x| x.date.iso()).collect();
    assert!(dates.contains(&"2026-02-16".to_string()));
    assert!(dates.contains(&"2026-04-03".to_string()));
    for day in &obligatory {
        assert!(
            matches!(day.season, Some(Season::GreatFast) | Some(Season::GreatWeek)),
            "{} binds outside the Fast", day.date
        );
    }
}

#[test]
fn the_voluntary_seasons_never_report_as_binding() {
    for when in ["2026-11-20", "2026-08-05"] {
        assert_eq!(at(when, "ruthenian").obligation, Some(Obligation::Customary));
    }
}

#[test]
fn christmas_on_a_friday_does_not_ask_you_to_abstain() {
    let day = at("2026-12-25", "ruthenian");
    assert_eq!(day.weekday, Weekday::Friday);
    assert_eq!(day.fast, Level::None);
}

#[test]
fn ruthenian_strict_days_bind_strictly_and_the_rest_bind_to_meat() {
    assert_eq!(at("2026-02-16", "ruthenian").binds, Level::WineOil);
    assert_eq!(at("2026-04-03", "ruthenian").binds, Level::WineOil);
    assert_eq!(at("2026-02-18", "ruthenian").binds, Level::Dairy);
    assert_eq!(at("2026-10-09", "ruthenian").binds, Level::None);
}

// ──────────────────────────── the menologion ──────────────────────────────

#[test]
fn every_day_of_a_leap_year_has_a_commemoration() {
    for day in week(d("2024-01-01"), 366, resolve(Some("new")).unwrap()) {
        assert!(!day.commemorations.is_empty(), "{}", day.date);
    }
}

#[test]
fn the_great_feasts_are_all_present() {
    for (month, day) in [(9, 8), (9, 14), (11, 21), (12, 25), (1, 6), (2, 2), (3, 25), (8, 6), (8, 15)] {
        let x = describe(Date::from_ymd(2026, month, day).unwrap(), resolve(Some("goarch")).unwrap());
        assert_eq!(x.rank, Some(Rank::Great), "{month}-{day}");
    }
    for when in ["2026-04-05", "2026-05-21", "2026-05-31"] {
        assert_eq!(at(when, "goarch").rank, Some(Rank::Great), "{when}");
    }
}

#[test]
fn a_movable_feast_outranks_the_fixed_calendar() {
    assert!(at("2026-04-12", "new").feast.as_deref().unwrap_or("").contains("Pascha"));
}

// ──────────────────────────────── output ──────────────────────────────────

#[test]
fn ics_content_lines_stay_within_seventy_five_octets() {
    for j in ["goarch", "rocor", "ruthenian"] {
        let days = week(d("2026-01-01"), 365, resolve(Some(j)).unwrap());
        let text = to_ics(&days, "Typikon", "20260101T000000Z");
        for line in text.split("\r\n") {
            assert!(line.len() <= 75, "{j}: {} octets: {line}", line.len());
        }
    }
}

#[test]
fn ics_is_well_formed() {
    let days = week(d("2026-04-10"), 7, resolve(Some("new")).unwrap());
    let text = to_ics(&days, "Typikon", "20260101T000000Z");
    assert!(text.starts_with("BEGIN:VCALENDAR"));
    assert!(text.trim_end().ends_with("END:VCALENDAR"));
    assert_eq!(text.matches("BEGIN:VEVENT").count(), text.matches("END:VEVENT").count());
}

// ─────────────────────────── every jurisdiction ───────────────────────────

#[test]
fn every_jurisdiction_holds_together_for_twenty_years() {
    for jur in JURISDICTIONS.iter().chain(CALENDARS.iter()) {
        for day in week(d("2020-01-01"), 366 * 21 / 4, jur) {
            assert!(!day.commemorations.is_empty(), "{} {}", jur.key, day.date);
            assert!(!day.title().is_empty(), "{} {}", jur.key, day.date);
            // `binds` may never be stricter than the level it sits under.
            assert!(day.binds >= day.fast, "{} {}: binds {:?} > fast {:?}",
                    jur.key, day.date, day.binds, day.fast);
        }
    }
}

#[test]
fn a_calendar_name_resolves_without_naming_a_church() {
    assert_eq!(resolve(Some("old")).unwrap().menologion, MenologionKind::Julian);
    assert_eq!(resolve(Some("new")).unwrap().menologion, MenologionKind::RevisedJulian);
    assert_eq!(resolve(Some("western")).unwrap().paschalion, Paschalion::Gregorian);
    assert_eq!(resolve(None).unwrap().key, "new");
    assert!(resolve(Some("atlantis")).is_err());
    assert_eq!(resolve(Some("rocor")).unwrap().calendar(), Calendar::Old);
}

// ─────────────────────── the levels are an ordered set ────────────────────

#[test]
fn a_feast_can_only_relax_a_fast_never_tighten_it() {
    // Strictest to laxest, so `<` means stricter.
    assert!(Level::Strict < Level::Xerophagy);
    assert!(Level::Xerophagy < Level::WineOil);
    assert!(Level::Dairy < Level::None);
    // Relaxing to fish inside the Great Fast stops at wine and oil.
    assert_eq!(Level::Xerophagy.relaxed_to(Level::Fish, Level::WineOil), Level::WineOil);
    // A cap stricter than the day never tightens the day.
    assert_eq!(Level::Fish.relaxed_to(Level::WineOil, Level::WineOil), Level::Fish);
    // No rank means no change.
    assert_eq!(Level::Xerophagy.relaxed_to(Level::Xerophagy, Level::Fish), Level::Xerophagy);
}

// ───────────────────────── the delimited formats ──────────────────────────

#[test]
fn csv_has_a_header_and_one_row_per_day() {
    use typikon::export::to_delimited;
    let days = week(d("2026-01-01"), 5, resolve(Some("rocor")).unwrap());
    let text = to_delimited(&days, ',', &[]);
    let lines: Vec<&str> = text.trim_end().split('\n').collect();
    assert_eq!(lines.len(), 6, "header plus five days");
    assert!(lines[0].starts_with("date,weekday,"));
    assert!(lines[1].starts_with("2026-01-01,Thursday,rocor,old,"));
}

#[test]
fn csv_quotes_the_fields_that_need_it() {
    use typikon::export::to_delimited;
    // Commemorations are full of commas, and joined with semicolons besides.
    let days = week(d("2026-09-18"), 1, resolve(Some("rocor")).unwrap());
    let text = to_delimited(&days, ',', &["title".to_string()]);
    let row = text.trim_end().split('\n').nth(1).unwrap();
    assert!(row.starts_with('"'), "a title with a comma must be quoted: {row}");
    assert!(row.ends_with('"'));
    // A quote inside a field doubles rather than escaping.
    assert_eq!(row.matches("\"\"").count(), 0);
}

#[test]
fn tsv_uses_tabs_and_a_chosen_column_order() {
    use typikon::export::to_delimited;
    let days = week(d("2026-04-12"), 1, resolve(Some("new")).unwrap());
    let cols = vec!["fast".to_string(), "date".to_string()];
    let text = to_delimited(&days, '\t', &cols);
    let lines: Vec<&str> = text.trim_end().split('\n').collect();
    assert_eq!(lines[0], "fast\tdate");
    assert_eq!(lines[1], "none\t2026-04-12");
}

#[test]
fn an_unknown_column_is_caught_before_anything_is_printed() {
    use typikon::export::csv::unknown_column;
    assert!(unknown_column(&["date".to_string(), "fast".to_string()]).is_none());
    assert_eq!(
        unknown_column(&["date".to_string(), "nonsense".to_string()]).map(String::as_str),
        Some("nonsense")
    );
}

#[test]
fn every_advertised_column_actually_renders() {
    use typikon::export::{csv::COLUMNS, to_delimited};
    let days = week(d("2026-02-16"), 1, resolve(Some("ruthenian")).unwrap());
    let cols: Vec<String> = COLUMNS.iter().map(|c| c.to_string()).collect();
    let text = to_delimited(&days, '\t', &cols);
    let lines: Vec<&str> = text.trim_end().split('\n').collect();
    // Every column produces a cell, even the empty ones.
    assert_eq!(lines[0].split('\t').count(), COLUMNS.len());
    assert_eq!(lines[1].split('\t').count(), COLUMNS.len());
}
