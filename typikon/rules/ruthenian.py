"""Byzantine Catholic particular law, alongside the tradition it sits on.

The Eastern Catholic churches differ from the Orthodox here in a way worth
being exact about: they have *law* on fasting, and it requires very much less
than the Typikon describes. Under current Ruthenian particular law the Church
binds only:

  - strict abstinence — no meat, fish, eggs or dairy — on the first day of the
    Great Fast and on Great and Holy Friday;
  - abstinence from meat on the Wednesdays and Fridays of the Great Fast.

Everything else — the Apostles', Dormition and Nativity Fasts, the fuller
keeping of the Great Fast, ordinary Fridays through the year — is encouraged
and traditional and does **not** bind. So this module reports two things at
once: the level at which the day is kept by tradition, and the minimum that
actually obliges. A calendar that says "obligatory" where nothing obliges is
worse than no calendar; one that reports only the binding minimum makes the
obligatory days look laxer than the voluntary ones beside them.

The three degrees this law knows map exactly onto the shared vocabulary:
its strict abstinence is ``wine_oil`` (it does not regulate oil or wine) and
its simple abstinence is ``dairy`` (meat alone is set aside).

Source: the Byzantine Catholic Metropolitan Church of Pittsburgh, whose
published rule on fasting and abstinence is the basis for the obligations
above. Other Eastern Catholic churches legislate their own; where this library
maps them onto this ruleset it is an approximation, and the jurisdiction entry
says so.
"""

import datetime as dt

from ..fasting import (Fast, WINE_OIL, DAIRY, NONE, OBLIGATORY, CUSTOMARY)
from ..seasons import (GREAT_FAST, GREAT_WEEK, BRIGHT_WEEK, CHEESEFARE_WEEK,
                       APOSTLES, DORMITION, NATIVITY_FAST, PENTECOST_WEEK,
                       CHRISTMASTIDE)

MON, TUE, WED, THU, FRI, SAT, SUN = range(7)

VOCABULARY = {
    WINE_OIL: ("no meat, fish, eggs or dairy", "\U0001f96c"),
    DAIRY:    ("no meat", "\U0001f41f"),
    NONE:     ("no fast", "\U0001f356"),
}


def _off_weekend(day):
    """Vigil fasts are kept on the preceding Friday when they land on a weekend.

    "If the vigils of Christmas or Theophany fall on Saturday or Sunday, the
    fasting and abstinence is done on the previous Friday."
    """
    if day.weekday() == SAT:
        return day - dt.timedelta(1)
    if day.weekday() == SUN:
        return day - dt.timedelta(2)
    return day


def fast_for(ctx):
    md, wd, mv = ctx.fixed, ctx.weekday, ctx.mv
    season = ctx.season

    # ── the two days the Church binds strictly
    if ctx.civil == mv["clean_monday"]:
        return Fast(WINE_OIL, "the first day of the Great Fast", OBLIGATORY, WINE_OIL)
    if ctx.civil == mv["great_friday"]:
        return Fast(WINE_OIL, "Great and Holy Friday", OBLIGATORY, WINE_OIL)

    # ── and the days it binds to abstinence from meat. Kept strictly by
    #    tradition; binding only as far as meat.
    if season == GREAT_FAST and wd in (WED, FRI):
        return Fast(WINE_OIL, "Wednesday and Friday of the Great Fast",
                    OBLIGATORY, DAIRY)

    # ── fixed vigils and the Exaltation, moved off a weekend where the rule says
    for when, label in (
        (ctx.civil_of_fixed(9, 14), "the Exaltation of the Holy Cross"),
        (_off_weekend(ctx.civil_of_fixed(12, 24)), "the vigil of the Nativity"),
        (_off_weekend(ctx.civil_of_fixed(1, 5)), "the vigil of Theophany"),
    ):
        if ctx.civil == when:
            return Fast(WINE_OIL, label, CUSTOMARY)

    # ── the fast-free stretches, which have to be settled before the ordinary
    #    Wednesday and Friday rule below or they get overridden by it. Christmas
    #    Day 2026 is a Friday, and telling you to abstain from meat on it would
    #    be plainly wrong.
    if season == BRIGHT_WEEK:
        return Fast(NONE, "Bright Week — no fasting", None)
    if season == PENTECOST_WEEK:
        return Fast(NONE, "the week after Pentecost — no fasting", None)
    if season == CHRISTMASTIDE:
        return Fast(NONE, "Christmastide — no fasting", None)
    if mv["publican_pharisee"] <= ctx.civil <= mv["publican_pharisee"] + dt.timedelta(6):
        return Fast(NONE, "fast-free week (the Publican and the Pharisee)", None)

    # ── the rest of the Great Fast and Great Week
    if season in (GREAT_FAST, GREAT_WEEK):
        if wd in (SAT, SUN):
            return Fast(DAIRY, season, CUSTOMARY)
        return Fast(WINE_OIL, season, CUSTOMARY)

    if season == CHEESEFARE_WEEK:
        return Fast(DAIRY, "Cheesefare Week", CUSTOMARY)

    # ── the three voluntary seasons
    if season in (APOSTLES, DORMITION, NATIVITY_FAST):
        return Fast(DAIRY, season, CUSTOMARY)

    # ── a great feast releases the ordinary weekday abstinence
    if wd in (WED, FRI):
        if ctx.rank == "great":
            return Fast(NONE, f"{ctx.feast} — the fast is released", None)
        return Fast(DAIRY, "Friday" if wd == FRI else "Wednesday", CUSTOMARY)

    return Fast(NONE, "", None)


RULE = {
    "name": "ruthenian",
    "title": "Ruthenian particular law",
    "vocabulary": VOCABULARY,
    "fast_for": fast_for,
    "has_obligation": True,
}
