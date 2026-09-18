"""Which season of the year a day falls in.

The movable seasons are counted from Pascha and so are compared as civil
dates. The fixed ones — the Nativity and Dormition Fasts, the fast-free days
after the Nativity — are compared in **ecclesiastical** space, against the date
the Church calls the day. That is the whole trick that lets one set of rules
serve the old and new calendars at once: the Nativity Fast begins on 15
November in both, and only the mornings differ.
"""

from .calendars import fixed_between

__all__ = ["GREAT_FAST", "GREAT_WEEK", "BRIGHT_WEEK", "CHEESEFARE_WEEK",
           "APOSTLES", "DORMITION", "NATIVITY_FAST", "season_for", "FASTING_SEASONS"]

GREAT_FAST = "Great Fast"
GREAT_WEEK = "Great and Holy Week"
BRIGHT_WEEK = "Bright Week"
PRE_LENTEN = "Triodion"
CHEESEFARE_WEEK = "Cheesefare Week"
APOSTLES = "Apostles' Fast"
DORMITION = "Dormition Fast"
NATIVITY_FAST = "Nativity Fast"
PENTECOST_WEEK = "Week after Pentecost"
CHRISTMASTIDE = "Christmastide"

FASTING_SEASONS = (GREAT_FAST, GREAT_WEEK, CHEESEFARE_WEEK,
                   APOSTLES, DORMITION, NATIVITY_FAST)


def season_for(civil, fixed_md, mv, apostles_end):
    """The season a day belongs to, or None for ordinary time.

    ``fixed_md`` is the ecclesiastical (month, day). ``apostles_end`` is the
    civil date of 28 June reckoned in this jurisdiction's menologion — the eve
    of Ss Peter and Paul, where the Apostles' Fast stops.
    """
    # Movable first: Great Week outranks the Great Fast it follows, and Bright
    # Week outranks everything.
    if mv["pascha"] <= civil <= mv["bright_week_end"]:
        return BRIGHT_WEEK
    if mv["great_week_start"] <= civil <= mv["holy_saturday"]:
        return GREAT_WEEK
    if mv["clean_monday"] <= civil <= mv["great_fast_end"]:
        return GREAT_FAST
    if mv["cheesefare_week_start"] <= civil <= mv["cheesefare"]:
        return CHEESEFARE_WEEK
    if mv["publican_pharisee"] <= civil < mv["cheesefare_week_start"]:
        return PRE_LENTEN
    if mv["pentecost"] <= civil <= mv["pentecost_week_end"]:
        return PENTECOST_WEEK

    # The Apostles' Fast can be empty in a year with a late Pascha, which the
    # comparison handles without a special case.
    if mv["apostles_start"] <= civil <= apostles_end:
        return APOSTLES

    # Fixed seasons, in ecclesiastical space.
    if fixed_between(fixed_md, (8, 1), (8, 14)):
        return DORMITION
    if fixed_between(fixed_md, (11, 15), (12, 24)):
        return NATIVITY_FAST
    if fixed_between(fixed_md, (12, 25), (1, 4)):
        return CHRISTMASTIDE
    return None
