"""A single day, fully reckoned.

``Day`` is what everything else in the library produces and what every
front end consumes. It carries the civil date, the ecclesiastical date, the
season, the commemorations and the fast — and it carries the jurisdiction that
produced it, so a caller can always ask by whose reckoning.
"""

import datetime as dt
from dataclasses import dataclass, field

from . import menologion as _menologion
from . import rules as _rules
from .calendars import civil_of_fixed, fixed_of_civil, julian_offset
from .fasting import NONE, OBLIGATORY
from .jurisdictions import resolve
from .paschalion import movable_cycle
from .seasons import season_for

__all__ = ["Day", "describe", "week", "year_of"]


@dataclass
class _Context:
    """What a fasting rule is handed. Deliberately small."""
    civil: dt.date
    fixed: tuple                 # ecclesiastical (month, day)
    fixed_year: int
    weekday: int                 # 0 = Monday
    mv: dict
    season: str
    feast: str
    rank: str
    menologion_kind: str

    def civil_of_fixed(self, month, day):
        """The civil date of an ecclesiastical date in this day's church year."""
        return civil_of_fixed(self.fixed_year, month, day, self.menologion_kind)


@dataclass
class Day:
    date: dt.date
    weekday: str
    jurisdiction: str
    calendar: str
    church_date: str             # the ecclesiastical date, ISO-ish
    season: str = None
    cycle: str = None            # forefeast / afterfeast / leave-taking
    feast: str = None
    rank: str = None
    commemorations: list = field(default_factory=list)
    fast: str = NONE
    fast_label: str = ""
    emoji: str = ""
    reason: str = ""
    obligation: str = None
    binds: str = NONE
    pascha: dt.date = None
    days_to_pascha: int = 0
    julian_offset: int = 0

    @property
    def is_obligatory(self):
        return self.obligation == OBLIGATORY

    @property
    def title(self):
        """What to put on one line: the feast if there is one, else the day's
        principal commemoration."""
        if self.feast:
            return self.feast
        return self.commemorations[0] if self.commemorations else ""

    def to_dict(self):
        out = {
            "date": self.date.isoformat(),
            "weekday": self.weekday,
            "jurisdiction": self.jurisdiction,
            "calendar": self.calendar,
            "churchDate": self.church_date,
            "season": self.season,
            "cycle": self.cycle,
            "feast": self.feast,
            "rank": self.rank,
            "title": self.title,
            "commemorations": self.commemorations,
            "fast": self.fast,
            "fastLabel": self.fast_label,
            "emoji": self.emoji,
            "reason": self.reason,
            "obligation": self.obligation,
            "binds": self.binds,
            "pascha": self.pascha.isoformat() if self.pascha else None,
            "daysToPascha": self.days_to_pascha,
            "julianOffset": self.julian_offset,
        }
        return out


_WEEKDAYS = ("Monday", "Tuesday", "Wednesday", "Thursday", "Friday",
             "Saturday", "Sunday")
_MONTHS = ("January", "February", "March", "April", "May", "June", "July",
           "August", "September", "October", "November", "December")


def describe(date, jurisdiction=None):
    """Reckon one civil date in one jurisdiction."""
    jur = jurisdiction if hasattr(jurisdiction, "paschalion") else resolve(jurisdiction)
    rule = _rules.get(jur.rule)
    men = _menologion.for_jurisdiction(jur)

    fy, fm, fd = fixed_of_civil(date, jur.menologion)
    mv = movable_cycle(date.year, jur.paschalion)
    apostles_end = civil_of_fixed(date.year, 6, 28, jur.menologion)
    season = season_for(date, (fm, fd), mv, apostles_end)

    entry = men.get(fm, fd)
    commemorations = list(entry.get("commemorations", []))
    rank = entry.get("rank")
    cycle = entry.get("cycle")

    # A movable feast outranks whatever the fixed calendar holds that day.
    mov_title, mov_rank = _menologion.movable_feast_for(date, mv)
    if mov_title:
        feast = mov_title
        rank = mov_rank or rank
    else:
        feast = commemorations[0] if rank in ("great", "vigil") else None

    ctx = _Context(
        civil=date, fixed=(fm, fd), fixed_year=fy, weekday=date.weekday(),
        mv=mv, season=season, feast=feast or (commemorations[0] if commemorations else ""),
        rank=rank, menologion_kind=jur.menologion,
    )
    fast = rule["fast_for"](ctx)
    label, emoji = rule["vocabulary"].get(fast.level, (fast.level, ""))

    return Day(
        date=date,
        weekday=_WEEKDAYS[date.weekday()],
        jurisdiction=jur.key,
        calendar=jur.calendar,
        church_date=f"{fd} {_MONTHS[fm - 1]}",
        season=season,
        cycle=cycle,
        feast=feast,
        rank=rank,
        commemorations=commemorations,
        fast=fast.level,
        fast_label=label,
        emoji=emoji,
        reason=fast.reason,
        obligation=fast.obligation,
        binds=fast.binds,
        pascha=mv["pascha"],
        days_to_pascha=(mv["pascha"] - date).days,
        julian_offset=julian_offset(date),
    )


def week(start, days=7, jurisdiction=None):
    """`days` consecutive days from `start`."""
    return [describe(start + dt.timedelta(n), jurisdiction)
            for n in range(max(1, days))]


def year_of(year, jurisdiction=None):
    """Every key date of a year, for checking against a published calendar."""
    jur = jurisdiction if hasattr(jurisdiction, "paschalion") else resolve(jurisdiction)
    mv = movable_cycle(year, jur.paschalion)
    mv["apostles_end"] = civil_of_fixed(year, 6, 28, jur.menologion)
    mv["nativity"] = civil_of_fixed(year, 12, 25, jur.menologion)
    mv["theophany"] = civil_of_fixed(year, 1, 6, jur.menologion)
    mv["dormition"] = civil_of_fixed(year, 8, 15, jur.menologion)
    return mv
