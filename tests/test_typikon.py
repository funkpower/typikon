"""Pinned against published calendars and against dates whose answer is not
open to interpretation."""

import datetime as dt

import pytest

from typikon import describe, gregorian_pascha, julian_pascha, resolve
from typikon.calendars import from_julian, julian_offset, to_julian
from typikon.day import week
from typikon.export import to_ics, to_json
from typikon.fasting import (DAIRY, FISH, NONE, OBLIGATORY, STRICT, WINE_OIL,
                             XEROPHAGY)


# ───────────────────────────── the paschalion ─────────────────────────────

@pytest.mark.parametrize("year,expected", [
    (2024, "2024-05-05"), (2025, "2025-04-20"), (2026, "2026-04-12"),
    (2027, "2027-05-02"), (2028, "2028-04-16"), (2030, "2030-04-28"),
])
def test_julian_pascha(year, expected):
    assert julian_pascha(year).isoformat() == expected


@pytest.mark.parametrize("year,expected", [
    (2024, "2024-03-31"), (2025, "2025-04-20"), (2026, "2026-04-05"),
    (2027, "2027-03-28"), (2028, "2028-04-16"),
])
def test_gregorian_pascha(year, expected):
    assert gregorian_pascha(year).isoformat() == expected


def test_the_two_paschalions_agree_in_2025_and_differ_in_2026():
    assert julian_pascha(2025) == gregorian_pascha(2025)
    assert (julian_pascha(2026) - gregorian_pascha(2026)).days == 7


# ──────────────────────────── calendar drift ──────────────────────────────

def test_julian_lags_thirteen_days_now_and_fourteen_after_2100():
    assert julian_offset(dt.date(2026, 9, 18)) == 13
    assert to_julian(dt.date(2026, 9, 18)) == (2026, 9, 5)
    # Julian 2100 is a leap year and Gregorian 2100 is not, so the gap
    # widens the day after the Julian 29 February — 15 March by the civil
    # calendar. Nothing here hardcodes it.
    assert julian_offset(dt.date(2100, 3, 14)) == 13
    assert julian_offset(dt.date(2100, 3, 15)) == 14


def test_old_calendar_nativity_falls_on_7_january():
    assert from_julian(2026, 12, 25) == dt.date(2027, 1, 7)


def test_old_and_new_disagree_about_the_day_not_the_date():
    old = describe(dt.date(2027, 1, 7), "rocor")
    new = describe(dt.date(2026, 12, 25), "goarch")
    assert old.church_date == new.church_date == "25 December"
    assert old.feast == new.feast
    assert old.rank == new.rank == "great"


# ───────────────────────── the Orthodox fast ──────────────────────────────

def test_great_fast_weekdays_are_xerophagy_and_weekends_wine_and_oil():
    # 2026: Orthodox Pascha 12 April, so Clean Monday is 23 February.
    assert describe(dt.date(2026, 2, 25), "new").fast == XEROPHAGY   # Wednesday
    assert describe(dt.date(2026, 2, 28), "new").fast == WINE_OIL    # Saturday
    assert describe(dt.date(2026, 3, 1), "new").fast == WINE_OIL     # Sunday


def test_the_two_strictest_days_of_the_year():
    assert describe(dt.date(2026, 2, 23), "new").fast == STRICT      # Clean Monday
    assert describe(dt.date(2026, 4, 10), "new").fast == STRICT      # Great Friday
    # The one Saturday kept without oil.
    assert describe(dt.date(2026, 4, 11), "new").fast == XEROPHAGY


def test_annunciation_and_palm_sunday_admit_fish_inside_the_great_fast():
    ann = describe(dt.date(2026, 3, 25), "new")
    assert ann.season == "Great Fast" and ann.fast == FISH
    assert describe(dt.date(2026, 4, 5), "new").fast == FISH         # Palm Sunday


def test_cheesefare_week_keeps_dairy_on_wednesday_and_friday():
    d = describe(dt.date(2026, 2, 18), "new")                        # Wednesday
    assert d.season == "Cheesefare Week" and d.fast == DAIRY


def test_the_fast_free_stretches():
    assert describe(dt.date(2026, 4, 17), "new").fast == NONE        # Bright Friday
    assert describe(dt.date(2026, 1, 2), "new").fast == NONE         # Christmastide
    assert describe(dt.date(2026, 6, 3), "new").fast == NONE         # week after Pentecost
    # The week after the Publican and the Pharisee, kept fast-free precisely
    # because the Pharisee boasted of fasting twice a week.
    assert describe(dt.date(2026, 2, 6), "new").fast == NONE


def test_nativity_fast_tightens_for_its_last_five_days():
    # A Saturday before 20 December admits fish; one after it does not.
    assert describe(dt.date(2026, 12, 12), "new").fast == FISH
    assert describe(dt.date(2026, 12, 26), "new").season != "Nativity Fast"
    late = describe(dt.date(2026, 12, 19), "new")
    assert late.fast == FISH
    assert describe(dt.date(2026, 12, 22), "new").fast == WINE_OIL   # Tuesday
    assert describe(dt.date(2026, 12, 24), "new").fast == XEROPHAGY  # the eve


def test_a_great_feast_lifts_an_ordinary_friday_to_fish():
    d = describe(dt.date(2026, 8, 6), "goarch")                      # Transfiguration
    assert d.weekday == "Thursday" and d.fast == FISH
    # 2026-01-01, the Circumcision, is a Thursday; take a great feast on a
    # Wednesday instead — the Meeting of our Lord, 2 February 2028.
    m = describe(dt.date(2028, 2, 2), "goarch")
    assert m.weekday == "Wednesday" and m.rank == "great" and m.fast == FISH


def test_ordinary_wednesday_and_friday_are_xerophagy():
    d = describe(dt.date(2026, 9, 4), "new")                         # a Friday
    assert d.season is None and d.rank is None and d.fast == XEROPHAGY


def test_a_polyeleos_saint_lifts_an_ordinary_friday_to_wine_and_oil():
    # 9 October 2026 is a Friday and the Apostle James son of Alphaeus. The
    # Typikon gives wine and oil for a saint of this rank, which is why the
    # published calendars mark so many Fridays that way.
    d = describe(dt.date(2026, 10, 9), "new")
    assert d.weekday == "Friday" and d.rank == "polyeleos"
    assert d.fast == WINE_OIL


def test_the_orthodox_rule_never_reports_an_obligation():
    days = week(dt.date(2026, 1, 1), 365, "new")
    assert not any(d.obligation == OBLIGATORY for d in days)


# ──────────────────── Byzantine Catholic particular law ───────────────────

def test_ruthenian_pascha_is_the_gregorian_one():
    assert describe(dt.date(2026, 6, 1), "ruthenian").pascha == dt.date(2026, 4, 5)


def test_ruthenian_clean_monday_is_16_february_2026():
    # As published by the Metropolitan Cantor Institute.
    assert describe(dt.date(2026, 2, 16), "ruthenian").reason.startswith(
        "the first day")


def test_only_four_kinds_of_day_bind_under_ruthenian_law():
    obligatory = [d for d in week(dt.date(2026, 1, 1), 365, "ruthenian")
                  if d.obligation == OBLIGATORY]
    # Clean Monday, Great Friday, and the Wednesdays and Fridays of the Fast.
    assert dt.date(2026, 2, 16) in [d.date for d in obligatory]
    assert dt.date(2026, 4, 3) in [d.date for d in obligatory]
    assert all(d.season in ("Great Fast", "Great and Holy Week")
               for d in obligatory)


def test_the_voluntary_seasons_never_report_as_binding():
    for when in (dt.date(2026, 11, 20), dt.date(2026, 8, 5)):
        assert describe(when, "ruthenian").obligation == "customary"


def test_christmas_on_a_friday_does_not_ask_you_to_abstain():
    d = describe(dt.date(2026, 12, 25), "ruthenian")
    assert d.weekday == "Friday" and d.fast == NONE


def test_ruthenian_strict_days_bind_strictly_and_the_rest_bind_to_meat():
    assert describe(dt.date(2026, 2, 16), "ruthenian").binds == WINE_OIL
    assert describe(dt.date(2026, 4, 3), "ruthenian").binds == WINE_OIL
    assert describe(dt.date(2026, 2, 18), "ruthenian").binds == DAIRY
    assert describe(dt.date(2026, 10, 9), "ruthenian").binds == NONE


# ──────────────────────────── the menologion ──────────────────────────────

def test_every_day_of_a_leap_year_has_a_commemoration():
    for d in week(dt.date(2024, 1, 1), 366, "new"):
        assert d.commemorations, d.date


def test_the_twelve_great_feasts_are_all_present():
    fixed = [(9, 8), (9, 14), (11, 21), (12, 25), (1, 6), (2, 2), (3, 25),
             (8, 6), (8, 15)]
    for month, day in fixed:
        d = describe(dt.date(2026, month, day), "goarch")
        assert d.rank == "great", (month, day)
    for when in (dt.date(2026, 4, 5), dt.date(2026, 5, 21), dt.date(2026, 5, 31)):
        assert describe(when, "goarch").rank == "great", when


def test_a_movable_feast_outranks_the_fixed_calendar():
    d = describe(dt.date(2026, 4, 12), "new")
    assert "Pascha" in d.feast


# ──────────────────────────────── output ──────────────────────────────────

def test_json_round_trips():
    import json
    days = week(dt.date(2026, 4, 10), 3, "new")
    parsed = json.loads(to_json(days))
    assert len(parsed["days"]) == 3
    assert parsed["days"][0]["fast"] == STRICT


def test_ics_is_well_formed_and_folds_long_lines():
    text = to_ics(week(dt.date(2026, 4, 10), 7, "new"))
    assert text.startswith("BEGIN:VCALENDAR")
    assert text.rstrip().endswith("END:VCALENDAR")
    assert text.count("BEGIN:VEVENT") == text.count("END:VEVENT")
    for line in text.split("\r\n"):
        assert len(line.encode()) < 200


# ─────────────────────────── every jurisdiction ───────────────────────────

@pytest.mark.parametrize("key", list(__import__("typikon").JURISDICTIONS))
def test_every_jurisdiction_reckons_a_whole_year_without_raising(key):
    days = week(dt.date(2026, 1, 1), 365, key)
    assert len(days) == 365
    assert all(d.fast for d in days)


def test_a_calendar_name_resolves_without_naming_a_church():
    assert resolve("old").menologion == "julian"
    assert resolve("new").menologion == "revised_julian"
    assert resolve("western").paschalion == "gregorian"
    assert resolve(None).key == "new"
