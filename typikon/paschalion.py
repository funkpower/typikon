"""Pascha, and the movable cycle that hangs off it.

Two reckonings are in use. The **Julian Paschalion** is what nearly every
Orthodox Church keeps: the old computus, worked in the Julian calendar and then
read off onto a civil date. The **Gregorian Paschalion** is the Western one,
kept by the Finnish Orthodox Church and by most Byzantine Catholic churches.

Neither is a variant of the other. They are separate tables that happen to
agree in some years — 2025 falls on 20 April by both — and can diverge by as
much as five weeks.
"""

import datetime as dt

from .calendars import from_julian

__all__ = ["julian_pascha", "gregorian_pascha", "pascha", "movable_cycle"]


def julian_pascha(year):
    """Pascha by the Julian Paschalion, as a civil (Gregorian) date.

    Meeus' Julian algorithm. It yields a date in the Julian calendar, which is
    then read onto the civil calendar — the step that is easy to forget and
    that puts the answer thirteen days out when it is.
    """
    a = year % 4
    b = year % 7
    c = year % 19
    d = (19 * c + 15) % 30
    e = (2 * a + 4 * b - d + 34) % 7
    month = (d + e + 114) // 31
    day = ((d + e + 114) % 31) + 1
    return from_julian(year, month, day)


def gregorian_pascha(year):
    """Pascha by the Gregorian Paschalion — the Western computus."""
    a = year % 19
    b, c = divmod(year, 100)
    d, e = divmod(b, 4)
    g = (8 * b + 13) // 25
    h = (19 * a + b - d - g + 15) % 30
    i, k = divmod(c, 4)
    ll = (32 + 2 * e + 2 * i - h - k) % 7
    m = (a + 11 * h + 19 * ll) // 433
    month = (h + ll - 7 * m + 90) // 25
    day = (h + ll - 7 * m + 33 * month + 19) % 32
    return dt.date(year, month, day)


def pascha(year, paschalion="julian"):
    if paschalion == "julian":
        return julian_pascha(year)
    if paschalion == "gregorian":
        return gregorian_pascha(year)
    raise ValueError(f"unknown paschalion: {paschalion!r}")


def movable_cycle(year, paschalion="julian"):
    """Every movable date the calendar needs, keyed off Pascha.

    All of these are counted in real days from Pascha, so they are civil dates
    already and need no calendar conversion — the distance from Pascha to
    Pentecost is fifty days in every reckoning.
    """
    p = pascha(year, paschalion)
    d = dt.timedelta

    return {
        "pascha": p,

        # ── the Triodion, counted backwards
        "publican_pharisee": p - d(70),
        "prodigal_son": p - d(63),
        "meatfare": p - d(56),          # last day meat is eaten
        "cheesefare": p - d(49),        # Forgiveness Sunday, last day of dairy
        "cheesefare_week_start": p - d(55),
        "clean_monday": p - d(48),      # the Great Fast begins
        "orthodoxy_sunday": p - d(42),
        "gregory_palamas": p - d(35),
        "cross_veneration": p - d(28),
        "john_climacus": p - d(21),
        "mary_of_egypt": p - d(14),
        "great_fast_end": p - d(9),     # Friday before Lazarus Saturday
        "lazarus_saturday": p - d(8),
        "palm_sunday": p - d(7),
        "great_week_start": p - d(6),
        "great_thursday": p - d(3),
        "great_friday": p - d(2),
        "holy_saturday": p - d(1),

        # ── the Pentecostarion, counted forwards
        "bright_week_end": p + d(6),
        "thomas_sunday": p + d(7),
        "myrrhbearers": p + d(14),
        "midfeast": p + d(24),
        "ascension": p + d(39),
        "fathers_first_council": p + d(42),
        "pentecost": p + d(49),
        "pentecost_week_end": p + d(55),
        "all_saints": p + d(56),

        # The Apostles' Fast runs from the Monday after All Saints to the eve
        # of Ss Peter and Paul. A late Pascha can leave it with no days at all,
        # which is correct rather than a bug — the fast is simply not kept that
        # year. Its end is a fixed ecclesiastical date, so the jurisdiction
        # resolves it; only the start belongs here.
        "apostles_start": p + d(57),
    }
