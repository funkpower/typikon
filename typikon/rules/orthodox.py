"""The Typikon's fast, as the Orthodox Church keeps it.

This is the discipline printed in the Typikon and reproduced, with small
variations, by every Orthodox jurisdiction's published calendar. It binds no
one in the canonical sense — the Orthodox Church legislates fasting as
tradition rather than as precept — so every day here is reported as customary.
That is not a softening. It is the difference between a rule of life and a law,
and the Eastern Catholic churches, which do have law about this, are handled in
their own module.

Where usages differ, this follows the common published rule and says so in the
reason. The known divergences:

  - **Great Thursday** is given wine and oil here. Some typika keep xerophagy.
  - **The Exaltation of the Cross** and the **Beheading of the Forerunner** are
    given wine and oil. Several jurisdictions print them as strict.
  - **Lazarus Saturday** is given wine and oil; caviar is the traditional
    addition, which this vocabulary has no level for.
  - **Monastic practice**, and the Holy Mountain in particular, is stricter
    than any of this throughout.

A calendar cannot settle those. Your bishop and your spiritual father can.
"""

import datetime as dt

from ..fasting import (Fast, STRICT, XEROPHAGY, WINE_OIL, FISH, DAIRY, NONE,
                       CUSTOMARY)
from ..seasons import (GREAT_FAST, GREAT_WEEK, BRIGHT_WEEK, CHEESEFARE_WEEK,
                       APOSTLES, DORMITION, NATIVITY_FAST, PENTECOST_WEEK,
                       CHRISTMASTIDE)

MON, TUE, WED, THU, FRI, SAT, SUN = range(7)

VOCABULARY = {
    STRICT:    ("no food until evening", "⛔"),
    XEROPHAGY: ("xerophagy — no oil or wine", "\U0001f96c"),
    WINE_OIL:  ("wine and oil permitted", "\U0001fad2"),
    FISH:      ("fish, wine and oil", "\U0001f41f"),
    DAIRY:     ("dairy and eggs; no meat", "\U0001f9c0"),
    NONE:      ("no fast", "\U0001f356"),
}


def _relaxed_by_feast(rank):
    """How far a feast lifts an ordinary Wednesday or Friday fast."""
    if rank == "great":
        return FISH
    if rank in ("vigil", "polyeleos"):
        return WINE_OIL
    return None


def fast_for(ctx):
    md, wd, mv = ctx.fixed, ctx.weekday, ctx.mv
    season = ctx.season
    weekend = wd in (SAT, SUN)

    # ── The fast-free stretches, settled before anything else can override
    #    them. A Friday inside Bright Week is not a fast day, and the ordinary
    #    Friday rule further down would say otherwise.
    if season == BRIGHT_WEEK:
        return Fast(NONE, "Bright Week — no fasting", None)
    if season == PENTECOST_WEEK:
        return Fast(NONE, "the week after Pentecost — no fasting", None)
    if season == CHRISTMASTIDE:
        return Fast(NONE, "the Nativity to the eve of Theophany", None)
    if mv["publican_pharisee"] <= ctx.civil <= mv["publican_pharisee"] + dt.timedelta(6):
        # Fast-free precisely because the Pharisee boasted of fasting twice a week.
        return Fast(NONE, "fast-free week (the Publican and the Pharisee)", None)

    # ── The Great Fast and Great Week
    if season in (GREAT_FAST, GREAT_WEEK):
        if ctx.civil == mv["clean_monday"]:
            return Fast(STRICT, "the first day of the Great Fast", CUSTOMARY)
        if ctx.civil == mv["great_friday"]:
            return Fast(STRICT, "Great and Holy Friday", CUSTOMARY)
        if ctx.civil == mv["holy_saturday"]:
            # The one Saturday of the year kept without oil.
            return Fast(XEROPHAGY, "Great and Holy Saturday", CUSTOMARY)
        if md == (3, 25):
            return Fast(FISH, "the Annunciation", CUSTOMARY)
        if ctx.civil == mv["great_thursday"]:
            return Fast(WINE_OIL, "Great and Holy Thursday", CUSTOMARY)
        if weekend:
            return Fast(WINE_OIL, f"Saturday and Sunday of the {season}", CUSTOMARY)
        return Fast(XEROPHAGY, f"a weekday of the {season}", CUSTOMARY)

    if ctx.civil == mv["palm_sunday"]:
        return Fast(FISH, "Palm Sunday", CUSTOMARY)
    if ctx.civil == mv["lazarus_saturday"]:
        return Fast(WINE_OIL, "Lazarus Saturday", CUSTOMARY)

    # ── Cheesefare: meat is set aside for good, everything else stays,
    #    Wednesday and Friday included.
    if season == CHEESEFARE_WEEK:
        return Fast(DAIRY, "Cheesefare Week", CUSTOMARY)
    if ctx.civil == mv["meatfare"]:
        return Fast(NONE, "Meatfare Sunday — the last day of meat", None)

    # ── The Nativity Fast, which tightens for its last five days
    if season == NATIVITY_FAST:
        if md == (12, 24):
            return Fast(XEROPHAGY, "the eve of the Nativity", CUSTOMARY)
        if md == (11, 21):
            return Fast(FISH, "the Entrance of the Theotokos", CUSTOMARY)
        if md >= (12, 20):
            if wd in (TUE, THU) or weekend:
                return Fast(WINE_OIL, "the forefeast of the Nativity", CUSTOMARY)
            return Fast(XEROPHAGY, "the forefeast of the Nativity", CUSTOMARY)
        if weekend:
            return Fast(FISH, "Saturday and Sunday of the Nativity Fast", CUSTOMARY)
        if wd in (TUE, THU):
            return Fast(WINE_OIL, "Tuesday and Thursday of the Nativity Fast", CUSTOMARY)
        return Fast(XEROPHAGY, "Monday, Wednesday and Friday of the Nativity Fast",
                    CUSTOMARY)

    # ── The Apostles' Fast, kept the same way
    if season == APOSTLES:
        if weekend:
            return Fast(FISH, "Saturday and Sunday of the Apostles' Fast", CUSTOMARY)
        if wd in (TUE, THU):
            return Fast(WINE_OIL, "Tuesday and Thursday of the Apostles' Fast", CUSTOMARY)
        return Fast(XEROPHAGY, "Monday, Wednesday and Friday of the Apostles' Fast",
                    CUSTOMARY)

    # ── The Dormition Fast, kept like the Great Fast
    if season == DORMITION:
        if md == (8, 6):
            return Fast(FISH, "the Transfiguration", CUSTOMARY)
        if weekend:
            return Fast(WINE_OIL, "Saturday and Sunday of the Dormition Fast", CUSTOMARY)
        return Fast(XEROPHAGY, "a weekday of the Dormition Fast", CUSTOMARY)

    # ── The three fixed fast days outside any season
    if md == (1, 5):
        return Fast(XEROPHAGY, "the eve of Theophany", CUSTOMARY)
    if md == (9, 14):
        return Fast(WINE_OIL, "the Exaltation of the Holy Cross", CUSTOMARY)
    if md == (8, 29):
        return Fast(WINE_OIL, "the Beheading of the Forerunner", CUSTOMARY)

    # ── Ordinary Wednesdays and Fridays, which a feast can lift
    if wd in (WED, FRI):
        relaxed = _relaxed_by_feast(ctx.rank)
        if relaxed:
            return Fast(relaxed, f"{ctx.feast} — the fast is relaxed", CUSTOMARY)
        return Fast(XEROPHAGY, "Wednesday and Friday", CUSTOMARY)

    return Fast(NONE, "", None)


RULE = {
    "name": "orthodox",
    "title": "the Typikon",
    "vocabulary": VOCABULARY,
    "fast_for": fast_for,
    # The Orthodox Church does not legislate the fast as precept, so nothing
    # here is reported as binding. See the module docstring.
    "has_obligation": False,
}
