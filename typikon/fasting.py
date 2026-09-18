"""What is kept on a day, and whether anything about it binds.

Six levels, ordered from strictest to laxest. They are the Typikon's own
distinctions, not a simplification of them: the difference between a day of
xerophagy and a day of wine and oil is the difference the books actually make,
and a calendar that reports both as "fasting" has thrown away the information
its reader wanted.

    strict      no food until evening
    xerophagy   no oil, no wine; no meat, fish, eggs or dairy
    wine_oil    wine and oil permitted; no meat, fish, eggs or dairy
    fish        fish, wine and oil permitted; no meat, eggs or dairy
    dairy       dairy, eggs and fish permitted; no meat
    none        no fast

The same six serve a tradition that makes fewer distinctions. Ruthenian
particular law knows three degrees, and each maps exactly onto one of these —
its "strict abstinence" is ``wine_oil`` (it does not regulate oil or wine), its
"abstinence" is ``dairy`` (meat alone is set aside). So a ruleset emits the
subset it uses and names those levels in its own vocabulary, rather than every
tradition being forced through one set of words.

**Level and obligation are separate questions.** A day can be kept strictly by
universal custom and bind no one, and that is the ordinary case in the Eastern
Catholic churches, whose particular law requires very little. Reporting the
binding minimum as the day's level makes the obligatory days look laxer than
the voluntary ones beside them — true of the obligation, false of the day. So
both are carried, and a caller that wants one must say which.
"""

from dataclasses import dataclass

__all__ = [
    "STRICT", "XEROPHAGY", "WINE_OIL", "FISH", "DAIRY", "NONE",
    "LEVELS", "stricter", "strictest", "laxest",
    "OBLIGATORY", "CUSTOMARY", "Fast",
]

STRICT = "strict"
XEROPHAGY = "xerophagy"
WINE_OIL = "wine_oil"
FISH = "fish"
DAIRY = "dairy"
NONE = "none"

# Strictest first. Index into this is the comparison.
LEVELS = (STRICT, XEROPHAGY, WINE_OIL, FISH, DAIRY, NONE)

OBLIGATORY = "obligatory"
CUSTOMARY = "customary"   # encouraged, traditional, not binding


def stricter(a, b):
    """True if `a` is a stricter level than `b`."""
    return LEVELS.index(a) < LEVELS.index(b)


def strictest(*levels):
    return min(levels, key=LEVELS.index)


def laxest(*levels):
    return max(levels, key=LEVELS.index)


@dataclass(frozen=True)
class Fast:
    """How a day is kept, and what about it binds.

    ``level`` is the day as the tradition keeps it. ``binds`` is what the
    Church actually requires, which for most jurisdictions is nothing — it is
    ``none`` unless a ruleset has particular law to point at. ``reason`` names
    the rule that decided it, so the answer can be argued with.
    """
    level: str
    reason: str = ""
    obligation: str = None       # OBLIGATORY, CUSTOMARY, or None for a free day
    binds: str = NONE            # the binding minimum, where one exists

    @property
    def is_fast(self):
        return self.level != NONE
