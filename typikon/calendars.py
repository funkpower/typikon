"""Conversion between the civil calendar and the calendar a church reckons in.

Every date in this library exists twice. There is the **civil** date — what a
phone says, always Gregorian — and the **ecclesiastical** date, the day the
Church calls it. For a church on the new calendar those are the same date. For
one on the old calendar they differ by the drift between the Julian and
Gregorian reckonings: thirteen days from 1900 to 2099, fourteen from 2100.

Keeping both is what makes a single menologion serve every jurisdiction. The
Nativity is 25 December in the Julian and the Gregorian books alike; the two
traditions disagree about which morning that is, not about which date it is. So
the fixed calendar is stored once, indexed by ecclesiastical date, and a
jurisdiction decides how a civil date maps onto it.

The drift is computed, never hardcoded, so nothing here breaks in 2100.
"""

import datetime as dt

__all__ = [
    "gregorian_to_jdn", "julian_to_jdn", "jdn_to_gregorian", "jdn_to_julian",
    "to_julian", "from_julian", "julian_offset",
]


def gregorian_to_jdn(year, month, day):
    """Julian Day Number for a proleptic Gregorian date."""
    a = (14 - month) // 12
    y = year + 4800 - a
    m = month + 12 * a - 3
    return (day + (153 * m + 2) // 5 + 365 * y
            + y // 4 - y // 100 + y // 400 - 32045)


def julian_to_jdn(year, month, day):
    """Julian Day Number for a proleptic Julian date."""
    a = (14 - month) // 12
    y = year + 4800 - a
    m = month + 12 * a - 3
    return day + (153 * m + 2) // 5 + 365 * y + y // 4 - 32083


def jdn_to_gregorian(jdn):
    a = jdn + 32044
    b, c = divmod(a, 146097)
    d = (4 * c + 3) // 146097
    e = c - (146097 * d) // 4
    f = (4 * e + 3) // 1461
    g = e - (1461 * f) // 4
    h = (5 * g + 2) // 153
    day = g - (153 * h + 2) // 5 + 1
    month = h + 3 - 12 * (h // 10)
    year = 400 * b + 100 * d + f - 4800 + h // 10
    return year, month, day


def jdn_to_julian(jdn):
    c = jdn + 32082
    d = (4 * c + 3) // 1461
    e = c - (1461 * d) // 4
    m = (5 * e + 2) // 153
    day = e - (153 * m + 2) // 5 + 1
    month = m + 3 - 12 * (m // 10)
    year = d - 4800 + m // 10
    return year, month, day


def to_julian(date):
    """The Julian calendar date of a civil (Gregorian) date.

    Returns a plain (year, month, day) triple rather than a ``date``, because
    29 February in the Julian calendar is a real day that ``datetime`` will not
    hold in a Gregorian common year.
    """
    return jdn_to_julian(gregorian_to_jdn(date.year, date.month, date.day))


def from_julian(year, month, day):
    """The civil (Gregorian) date of a Julian calendar date."""
    y, m, d = jdn_to_gregorian(julian_to_jdn(year, month, day))
    return dt.date(y, m, d)


def julian_offset(date):
    """How many days the Julian reckoning currently lags the Gregorian one.

    Thirteen for our lifetimes; fourteen from 14 March 2100. Reported rather
    than assumed so a caller can show it.
    """
    jy, jm, jd = to_julian(date)
    return (gregorian_to_jdn(date.year, date.month, date.day)
            - gregorian_to_jdn(jy, jm, jd))


def fixed_of_civil(date, menologion):
    """The ecclesiastical (year, month, day) a civil date is reckoned as.

    On the Revised Julian menologion this is the civil date itself — the two
    agree for every date until 2800, so the distinction costs nothing and stops
    mattering only long after anyone reading this.
    """
    if menologion == "julian":
        return to_julian(date)
    return (date.year, date.month, date.day)


def civil_of_fixed(year, month, day, menologion):
    """The civil date of an ecclesiastical date in a given ecclesiastical year."""
    if menologion == "julian":
        return from_julian(year, month, day)
    return dt.date(year, month, day)


def fixed_between(md, start, end):
    """Is (month, day) within an inclusive fixed-date range, wrapping the year?

    The Nativity Fast does not wrap; the fast-free stretch from the Nativity to
    the eve of Theophany does. Both are written the same way here.
    """
    if start <= end:
        return start <= md <= end
    return md >= start or md <= end
