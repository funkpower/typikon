"""typikon — the Orthodox liturgical day, computed rather than fetched.

    >>> import typikon
    >>> d = typikon.today("rocor")
    >>> d.church_date, d.fast, d.title

Everything is derived from the paschalion and the fixed calendar, so it needs
no network, cannot go stale, and answers for any year you ask about.
"""

import datetime as dt

from .day import Day, describe, week, year_of
from .fasting import (DAIRY, FISH, LEVELS, NONE, STRICT, WINE_OIL, XEROPHAGY,
                      Fast)
from .jurisdictions import CALENDARS, JURISDICTIONS, Jurisdiction, resolve
from .paschalion import gregorian_pascha, julian_pascha, movable_cycle, pascha

__version__ = "0.1.0"

__all__ = [
    "Day", "Fast", "Jurisdiction",
    "describe", "today", "week", "year_of", "resolve",
    "pascha", "julian_pascha", "gregorian_pascha", "movable_cycle",
    "JURISDICTIONS", "CALENDARS", "LEVELS",
    "STRICT", "XEROPHAGY", "WINE_OIL", "FISH", "DAIRY", "NONE",
]


def today(jurisdiction=None):
    return describe(dt.date.today(), jurisdiction)
