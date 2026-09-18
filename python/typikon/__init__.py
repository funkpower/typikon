"""The Orthodox liturgical day: feasts, seasons and the fast, computed rather than fetched.

Everything derives from the paschalion and the fixed calendar, so this needs no
network, cannot go stale, and answers for any year you ask about.

    >>> import typikon, datetime
    >>> day = typikon.describe(datetime.date(2026, 4, 12), jurisdiction='new')
    >>> day.feast
    'The Radiant and Life-giving Resurrection of our Lord — Holy Pascha'
    >>> day.fast, day.days_to_pascha
    ('none', 0)

The jurisdiction is given here because 12 April 2026 is Pascha on the Julian
paschalion and an ordinary Sunday on the Gregorian one. Called without one,
every function uses whatever `typikon --set-default` wrote, so the library and
the command line agree about the same date. Pass ``jurisdiction=`` to override
it; ``jurisdictions()`` lists the keys.
"""

from ._typikon import (
    Day,
    __version__,
    describe,
    jurisdictions,
    pascha,
    today,
    week,
)

__all__ = [
    "Day",
    "__version__",
    "describe",
    "jurisdictions",
    "pascha",
    "today",
    "week",
]
