//! Pinned against published calendars and against dates whose answer is not
//! open to interpretation.

use typikon::calendars::{Date, JulianDate, Weekday};
use typikon::day::{describe, week, year_of};
use typikon::export::{to_ics, to_ics_with, Ics};
use typikon::fasting::{Level, Obligation};
use typikon::jurisdictions::{resolve, Calendar, MenologionKind, CALENDARS, JURISDICTIONS};
use typikon::menologion::Rank;
use typikon::paschalion::{gregorian_pascha, julian_pascha, week_after_pentecost, Paschalion};
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
    assert_eq!(at("2026-02-16", "ruthenian").binds, Some(Level::WineOil));
    assert_eq!(at("2026-04-03", "ruthenian").binds, Some(Level::WineOil));
    assert_eq!(at("2026-02-18", "ruthenian").binds, Some(Level::Dairy));
    assert_eq!(at("2026-10-09", "ruthenian").binds, None);
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

#[test]
fn every_day_gives_every_day_an_event() {
    let days = week(d("2026-09-21"), 60, resolve(Some("ruthenian")).unwrap());
    let thin = to_ics(&days, "Typikon", "20260101T000000Z");
    let full = to_ics_with(
        &days,
        &Ics { name: "Typikon", stamp: "20260101T000000Z", every_day: true },
    );
    assert!(thin.matches("BEGIN:VEVENT").count() < 60, "the thinned calendar drops days");
    assert_eq!(full.matches("BEGIN:VEVENT").count(), 60);
    for line in full.split("\r\n") {
        assert!(line.len() <= 75, "{} octets: {line}", line.len());
    }
}

#[test]
fn an_event_carries_the_saints_and_the_fast() {
    // A ferial Wednesday of the Great Fast: no feast, a fast that binds, and
    // a saint the thinned calendar would keep but the old body did not name
    // until after the church date.
    let days = week(d("2027-03-17"), 1, resolve(Some("ruthenian")).unwrap());
    let text = to_ics_with(
        &days,
        &Ics { name: "Typikon", stamp: "20260101T000000Z", every_day: true },
    );
    let unfolded = text.replace("\r\n ", "");
    let description = unfolded
        .lines()
        .find(|l| l.starts_with("DESCRIPTION:"))
        .expect("an event with a description");
    assert!(description.contains("\u{2022} Our Venerable Father Alexis the Man of God"),
            "{description}");
    assert!(description.contains("Fast: no meat"), "{description}");
    assert!(description.contains("Obligatory."), "{description}");
    assert!(description.contains("Church date: 17 March"), "{description}");
}

// ────────────────────── the Sundays of the movable year ──────────────────────

#[test]
fn the_pentecostarion_names_all_of_its_sundays() {
    // Pascha 2027 is 28 March by the Gregorian paschalion and 2 May by the
    // Julian one, so the same Sundays are asked of both reckonings.
    for (jur, pascha) in [("ruthenian", "2027-03-28"), ("rocor", "2027-05-02")] {
        let pascha = d(pascha);
        for (offset, name) in [
            (7, "Thomas Sunday"),
            (14, "The Sunday of the Holy Myrrh-bearing Women"),
            (21, "The Sunday of the Paralytic"),
            (28, "The Sunday of the Samaritan Woman"),
            (35, "The Sunday of the Blind Man"),
            (42, "The Sunday of the Fathers of the First Council"),
        ] {
            let day = describe(pascha + offset, resolve(Some(jur)).unwrap());
            assert_eq!(day.weekday, Weekday::Sunday, "{jur} +{offset}");
            let named = day.title().starts_with(name)
                || day.commemorations.iter().any(|c| c.starts_with(name));
            assert!(named, "{jur} {} ({}) does not name {name}: {:?}",
                    day.date, offset, day.commemorations);
        }
    }
}

#[test]
fn a_vigil_rank_saint_keeps_the_day_and_the_sunday_joins_it() {
    // 25 April 2027 is both the Sunday of the Samaritan Woman and St Mark,
    // whose feast is of vigil rank. The apostle names the day; the Sunday is
    // commemorated on it.
    let day = at("2027-04-25", "ruthenian");
    assert_eq!(day.title(), "The Holy Apostle and Evangelist Mark");
    assert_eq!(day.rank, Some(Rank::Vigil));
    assert!(day.commemorations.iter().any(|c| c == "The Sunday of the Samaritan Woman"),
            "{:?}", day.commemorations);
}

#[test]
fn the_sundays_reckoned_from_a_fixed_feast_are_kept() {
    for (date, jur, name) in [
        ("2026-09-13", "ruthenian", "The Sunday before the Exaltation"),
        ("2026-09-20", "ruthenian", "The Sunday after the Exaltation"),
        ("2026-12-13", "ruthenian", "The Sunday of the Holy Forefathers"),
        ("2026-12-20", "ruthenian", "The Sunday before the Nativity \u{2014} the Holy Fathers"),
        ("2026-12-27", "ruthenian", "The Sunday after the Nativity"),
        ("2027-01-03", "ruthenian", "The Sunday before the Theophany"),
        ("2027-01-10", "ruthenian", "The Sunday after the Theophany"),
        // The old calendar keeps the same windows thirteen days later, which
        // is the whole point of reckoning them in ecclesiastical space.
        ("2027-01-03", "moscow", "The Sunday before the Nativity \u{2014} the Holy Fathers"),
        ("2027-01-10", "moscow", "The Sunday after the Nativity"),
        ("2027-01-17", "moscow", "The Sunday before the Theophany"),
    ] {
        let day = at(date, jur);
        assert!(day.commemorations.iter().any(|c| c == name),
                "{jur} {date}: {name} missing from {:?}", day.commemorations);
    }
}

#[test]
fn a_nativity_on_a_sunday_moves_the_sunday_after_it_to_the_26th() {
    // 25 December 2033 is a Sunday, so 26–31 December holds none and the
    // Typikon keeps the Sunday after the Nativity on the Monday.
    assert_eq!(at("2033-12-25", "ruthenian").weekday, Weekday::Sunday);
    let monday = at("2033-12-26", "ruthenian");
    assert_eq!(monday.weekday, Weekday::Monday);
    assert!(monday.commemorations.iter().any(|c| c == "The Sunday after the Nativity"),
            "{:?}", monday.commemorations);
    // And it is kept once, not twice.
    for n in 1..=6 {
        let day = describe(d("2033-12-26") + n, resolve(Some("ruthenian")).unwrap());
        assert!(!day.commemorations.iter().any(|c| c == "The Sunday after the Nativity"),
                "{}", day.date);
    }
}

#[test]
fn a_new_year_on_a_sunday_keeps_the_sunday_before_the_theophany_with_it() {
    let day = at("2034-01-01", "ruthenian");
    assert_eq!(day.weekday, Weekday::Sunday);
    assert!(day.commemorations.iter().any(|c| c == "The Sunday before the Theophany"),
            "{:?}", day.commemorations);
}

#[test]
fn no_sunday_of_the_year_is_named_twice() {
    for jur in JURISDICTIONS.iter().chain(CALENDARS.iter()) {
        for day in week(d("2024-01-01"), 366 * 12, jur) {
            let mut seen = day.commemorations.clone();
            seen.sort();
            let before = seen.len();
            seen.dedup();
            assert_eq!(seen.len(), before, "{} {}: {:?}", jur.key, day.date, day.commemorations);
        }
    }
}

// ───────────────────── the count of weeks from Pentecost ─────────────────────

#[test]
fn the_weeks_after_pentecost_are_counted_from_all_saints() {
    let p = Paschalion::Gregorian;
    // Pascha 2026 is 5 April, so Pentecost is 24 May and All Saints 31 May.
    assert_eq!(week_after_pentecost(d("2026-05-24"), p), None, "Pentecost itself");
    assert_eq!(week_after_pentecost(d("2026-05-25"), p), Some(1), "the Monday after");
    assert_eq!(week_after_pentecost(d("2026-05-31"), p), Some(1), "All Saints");
    assert_eq!(week_after_pentecost(d("2026-06-07"), p), Some(2));
    assert_eq!(week_after_pentecost(d("2026-09-27"), p), Some(18));
    // The count does not restart at the new year: it runs on from last
    // year's Pentecost until the Triodion opens.
    assert_eq!(week_after_pentecost(d("2027-01-10"), p), Some(33));
    assert_eq!(week_after_pentecost(d("2027-01-16"), p), Some(34));
    // 17 January 2027 is the Publican and the Pharisee, and the Triodion
    // names its own Sundays.
    assert_eq!(week_after_pentecost(d("2027-01-17"), p), None);
    assert_eq!(week_after_pentecost(d("2027-03-28"), p), None, "Pascha");
}

#[test]
fn the_week_is_worded_for_the_day_it_falls_on() {
    assert_eq!(at("2026-09-27", "ruthenian").week.as_deref(),
               Some("the 18th Sunday after Pentecost"));
    assert_eq!(at("2026-09-28", "ruthenian").week.as_deref(),
               Some("the 19th week after Pentecost"));
    // The Pentecostarion and the Triodion carry names instead of numbers.
    assert_eq!(at("2027-04-18", "ruthenian").week, None);
    assert_eq!(at("2027-02-14", "ruthenian").week, None);
}

#[test]
fn the_week_count_never_skips_or_repeats_a_number() {
    for jur in JURISDICTIONS.iter().chain(CALENDARS.iter()) {
        // The walk starts mid-count, so the first run it sees is not a run
        // that opened. Every later one is.
        let mut last: Option<i32> = None;
        let mut first_run = true;
        for day in week(d("2025-01-01"), 366 * 6, jur) {
            let Some(n) = week_after_pentecost(day.date, jur.paschalion) else {
                if last.is_some() {
                    first_run = false;
                }
                last = None;
                continue;
            };
            match last {
                Some(prev) => assert!(n == prev || n == prev + 1,
                                      "{} {}: {prev} then {n}", jur.key, day.date),
                None if !first_run => assert_eq!(n, 1,
                                      "{} {}: the count opens at one", jur.key, day.date),
                None => {}
            }
            last = Some(n);
        }
    }
}

// ─────────────────────────── every jurisdiction ───────────────────────────

#[test]
fn every_jurisdiction_holds_together_for_twenty_years() {
    for jur in JURISDICTIONS.iter().chain(CALENDARS.iter()) {
        for day in week(d("2020-01-01"), 366 * 21, jur) {
            assert!(!day.commemorations.is_empty(), "{} {}", jur.key, day.date);
            assert!(!day.title().is_empty(), "{} {}", jur.key, day.date);
            // `binds` may never be stricter than the level it sits under.
            if let Some(binds) = day.binds {
                assert!(binds >= day.fast, "{} {}: binds {:?} > fast {:?}",
                        jur.key, day.date, binds, day.fast);
            }
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

// ───────────────────────── Greek and Slavic usage ─────────────────────────

#[test]
fn the_protection_is_a_great_feast_in_slavic_usage_only() {
    use typikon::jurisdictions::{Jurisdiction, Tradition};
    let base = *resolve(Some("goarch")).unwrap();
    let slavic: &'static Jurisdiction =
        Box::leak(Box::new(Jurisdiction { tradition: Tradition::Slavic, ..base }));
    let greek: &'static Jurisdiction =
        Box::leak(Box::new(Jurisdiction { tradition: Tradition::Greek, ..base }));
    assert_eq!(describe(d("2026-10-01"), slavic).rank, Some(Rank::Great));
    assert_eq!(describe(d("2026-10-01"), greek).rank, Some(Rank::Vigil));
}

#[test]
fn greek_usage_admits_fish_in_the_nativity_fast_where_slavic_does_not() {
    // A Tuesday before 17 December.
    assert_eq!(at("2026-12-01", "rocor").fast, Level::WineOil);
    assert_eq!(at("2026-12-01", "goarch").fast, Level::Fish);
    // Wednesday is a fast day in both.
    assert!(at("2026-12-02", "goarch").fast <= Level::WineOil);
    // And after the 17th the Greek relaxation stops.
    assert_eq!(at("2026-12-22", "goarch").fast, Level::WineOil);
}

#[test]
fn a_jurisdiction_carries_the_usage_its_books_follow() {
    use typikon::jurisdictions::Tradition;
    for key in ["moscow", "rocor", "serbian", "oca", "ruthenian", "ukrainian"] {
        assert_eq!(resolve(Some(key)).unwrap().tradition, Tradition::Slavic, "{key}");
    }
    for key in ["goarch", "greece", "constantinople", "antiochian", "cyprus"] {
        assert_eq!(resolve(Some(key)).unwrap().tradition, Tradition::Greek, "{key}");
    }
}

#[test]
fn every_rank_has_a_mark_and_they_are_distinct() {
    let signs: Vec<&str> = [Rank::Great, Rank::Vigil, Rank::Polyeleos]
        .iter().map(|r| r.sign()).collect();
    assert!(signs.iter().all(|s| !s.is_empty()));
    assert_eq!(signs.iter().collect::<std::collections::HashSet<_>>().len(), 3);
}

#[test]
fn tradition_parses_the_names_people_use() {
    use typikon::jurisdictions::Tradition;
    assert_eq!(Tradition::parse("Slavic"), Some(Tradition::Slavic));
    assert_eq!(Tradition::parse("russian"), Some(Tradition::Slavic));
    assert_eq!(Tradition::parse("GREEK"), Some(Tradition::Greek));
    assert_eq!(Tradition::parse("coptic"), None);
}

// ─────────────── the law that binds vs the discipline shown ───────────────

fn ruthenian_at_typikon() -> &'static typikon::jurisdictions::Jurisdiction {
    use typikon::jurisdictions::{Jurisdiction, RuleKind};
    let base = *resolve(Some("ruthenian")).unwrap();
    Box::leak(Box::new(Jurisdiction { discipline: Some(RuleKind::Orthodox), ..base }))
}

#[test]
fn the_typikon_discipline_can_be_shown_without_losing_the_obligation() {
    let jur = ruthenian_at_typikon();
    // Clean Monday: the Typikon's total fast, still binding under particular law.
    let clean = describe(d("2026-02-16"), jur);
    assert_eq!(clean.fast, Level::Strict);
    assert_eq!(clean.obligation, Some(Obligation::Obligatory));
    assert_eq!(clean.binds, Some(Level::WineOil));

    // A Wednesday of the Great Fast: xerophagy kept, meat alone binding.
    let wed = describe(d("2026-02-18"), jur);
    assert_eq!(wed.fast, Level::Xerophagy);
    assert_eq!(wed.obligation, Some(Obligation::Obligatory));
    assert_eq!(wed.binds, Some(Level::Dairy));

    // A Thursday of the Great Fast: kept, and binding on no one.
    let thu = describe(d("2026-02-19"), jur);
    assert_eq!(thu.fast, Level::Xerophagy);
    assert_eq!(thu.obligation, Some(Obligation::Customary));
    assert_eq!(thu.binds, None);
}

#[test]
fn showing_the_typikon_never_invents_an_obligation() {
    let jur = ruthenian_at_typikon();
    let native = resolve(Some("ruthenian")).unwrap();
    for n in 0..365 {
        let day = describe(d("2026-01-01") + n, jur);
        let plain = describe(d("2026-01-01") + n, native);
        // Exactly the same days bind, whichever discipline is displayed.
        assert_eq!(day.is_obligatory(), plain.is_obligatory(), "{}", day.date);
        assert_eq!(day.binds, plain.binds, "{}", day.date);
    }
}

#[test]
fn the_typikon_discipline_is_never_laxer_than_the_law_it_reports() {
    let jur = ruthenian_at_typikon();
    for n in 0..365 {
        let day = describe(d("2026-01-01") + n, jur);
        if let Some(binds) = day.binds {
            assert!(binds >= day.fast, "{}: binds {:?} stricter than kept {:?}",
                    day.date, binds, day.fast);
        }
    }
}

#[test]
fn the_override_changes_nothing_for_a_jurisdiction_already_on_the_typikon() {
    use typikon::jurisdictions::{Jurisdiction, RuleKind};
    let base = *resolve(Some("rocor")).unwrap();
    let forced: &'static Jurisdiction =
        Box::leak(Box::new(Jurisdiction { discipline: Some(RuleKind::Orthodox), ..base }));
    for n in 0..200 {
        let a = describe(d("2026-01-01") + n, resolve(Some("rocor")).unwrap());
        let b = describe(d("2026-01-01") + n, forced);
        assert_eq!(a.fast, b.fast, "{}", a.date);
        assert_eq!(a.obligation, b.obligation, "{}", a.date);
    }
}

// ────────────────────────── what gets remembered ──────────────────────────

#[test]
fn the_config_round_trips_all_three_axes() {
    use typikon::config::{self, Saved};
    // A scratch XDG_CONFIG_HOME, so the test cannot touch a real one.
    let dir = std::env::temp_dir().join(format!("typikon-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    // SAFETY: single-threaded test, and the variable is restored below.
    let prev = std::env::var_os("XDG_CONFIG_HOME");
    unsafe { std::env::set_var("XDG_CONFIG_HOME", &dir) };

    config::write(&Saved {
        jurisdiction: Some("ruthenian".into()),
        tradition: Some("slavic".into()),
        typikon: true,
    })
    .unwrap();
    let back = config::saved();
    assert_eq!(back.jurisdiction.as_deref(), Some("ruthenian"));
    assert_eq!(back.tradition.as_deref(), Some("slavic"));
    assert!(back.typikon);

    // Remembering only a jurisdiction leaves the other two unset, rather than
    // inventing a default that would disagree with the flags.
    config::write(&Saved { jurisdiction: Some("rocor".into()), ..Default::default() }).unwrap();
    let plain = config::saved();
    assert_eq!(plain.jurisdiction.as_deref(), Some("rocor"));
    assert_eq!(plain.tradition, None);
    assert!(!plain.typikon);

    unsafe {
        match prev {
            Some(v) => std::env::set_var("XDG_CONFIG_HOME", v),
            None => std::env::remove_var("XDG_CONFIG_HOME"),
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
}

// ─────────────── no church is told another church's law binds ─────────────

#[test]
fn only_the_church_whose_law_was_sourced_reports_an_obligation() {
    use typikon::jurisdictions::{RuleKind, JURISDICTIONS};
    // Ruthenian particular law is sourced from the Metropolia of Pittsburgh.
    // No other jurisdiction may borrow it: telling a Melkite that fourteen
    // days a year bind him under a law that was never his is the error the
    // level/obligation split exists to prevent.
    for jur in JURISDICTIONS.iter() {
        if jur.rule == RuleKind::Ruthenian {
            assert_eq!(jur.key, "ruthenian", "{} borrows another church's law", jur.key);
        }
    }

    for key in ["melkite", "romanian-catholic", "ugcc"] {
        let jur = resolve(Some(key)).unwrap();
        let binding = week(d("2026-01-01"), 365, jur)
            .into_iter()
            .filter(|x| x.is_obligatory())
            .count();
        assert_eq!(binding, 0, "{key} claims {binding} binding days it has no source for");
        // And the entry has to say why nothing binds.
        assert!(jur.note.contains("not modelled"), "{key}: the note must say so");
    }

    // The one church whose law is sourced still reports it.
    let ruthenian = resolve(Some("ruthenian")).unwrap();
    let binding = week(d("2026-01-01"), 365, ruthenian)
        .into_iter()
        .filter(|x| x.is_obligatory())
        .count();
    assert_eq!(binding, 14);
}

// ───────────────────── the edges of the supported years ───────────────────

#[test]
fn dates_outside_the_supported_years_cannot_be_made() {
    use typikon::calendars::{MAX_YEAR, MIN_YEAR};
    assert!(Date::from_ymd(MIN_YEAR, 1, 1).is_some());
    assert!(Date::from_ymd(MAX_YEAR, 12, 31).is_some());
    assert!(Date::from_ymd(MIN_YEAR - 1, 12, 31).is_none());
    assert!(Date::from_ymd(MAX_YEAR + 1, 1, 1).is_none());
    assert!(Date::from_ymd(2026, 2, 30).is_none());
    assert!(Date::from_jdn(i32::MAX).is_none());
    assert!(Date::from_jdn(i32::MIN).is_none());
}

#[test]
fn a_walk_off_the_end_stops_rather_than_wrapping() {
    let jur = resolve(Some("new")).unwrap();
    // Every date emitted is a real one, and the year never leaves the range.
    let days = week(d("9999-12-29"), 10, jur);
    assert_eq!(days.len(), 3, "the walk should stop at 9999-12-31");
    for day in &days {
        assert_eq!(day.date.iso().len(), 10, "{}", day.date);
        assert!(day.date.year() <= 9999);
    }
    // And the other end.
    assert_eq!(week(d("0001-01-01") , 3, jur).len(), 3);
}

#[test]
fn date_arithmetic_saturates_instead_of_overflowing() {
    // In a release build an unchecked add wraps silently, which put a date in
    // antiquity and an ISO string that was not ten characters long.
    assert_eq!(Date::MAX + 1, Date::MAX);
    assert_eq!(Date::MIN - 1, Date::MIN);
    assert_eq!(Date::MAX + i32::MAX, Date::MAX);
    assert_eq!(Date::MIN - i32::MAX, Date::MIN);
    assert!(Date::MAX.checked_add(1).is_none());
    for d in [Date::MIN, Date::MAX] {
        assert_eq!(d.iso().len(), 10);
    }
}

/// The published description counts the churches, on PyPI and on npm, where
/// nobody editing this table will see it. Adding a jurisdiction without saying
/// so makes that description quietly wrong, so the count is pinned here and
/// the failure names the two places to correct.
#[test]
fn the_published_description_still_counts_the_churches_correctly() {
    let catholic = ["ruthenian", "ugcc", "melkite", "romanian-catholic"];
    let orthodox = JURISDICTIONS
        .iter()
        .filter(|j| !catholic.contains(&j.key))
        .count();
    let byzantine_catholic = JURISDICTIONS
        .iter()
        .filter(|j| catholic.contains(&j.key))
        .count();

    assert_eq!(
        (orthodox, byzantine_catholic, CALENDARS.len()),
        (20, 4, 3),
        "the counts moved; update `description` in Cargo.toml (PyPI) and in \
         npm/package.json.in (npm), and the README's opening"
    );
}
