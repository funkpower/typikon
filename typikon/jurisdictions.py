"""Who keeps which calendar, and by whose rule.

A jurisdiction is four independent choices, and conflating any two of them is
how calendar code usually goes wrong:

    paschalion   which table gives Pascha — julian or gregorian
    menologion   which calendar the fixed feasts are read on
    rule         which fasting discipline is kept
    menaia       which supplementary commemorations to load, if any

The pairings are not free. Almost every Orthodox Church keeps the Julian
Paschalion; what divides them is the menologion — the "old calendar" churches
read the fixed feasts on the Julian calendar too, the "new calendar" churches
read them on the Revised Julian, which agrees with the Gregorian for every date
until 2800. So a new-calendar church keeps Christmas with the West and Pascha
with the East, which is not a compromise but exactly what the 1923 congress
decided.

Only two bodies here use the Gregorian Paschalion: the Finnish Orthodox Church,
alone among the Orthodox, and the Byzantine Catholic churches.

Where a church is mid-transition or its parishes differ, the entry says so in
``note`` rather than pretending to a precision that does not exist.
"""

from dataclasses import dataclass, field

__all__ = ["Jurisdiction", "JURISDICTIONS", "CALENDARS", "resolve", "names"]


@dataclass(frozen=True)
class Jurisdiction:
    key: str
    name: str
    paschalion: str          # "julian" | "gregorian"
    menologion: str          # "julian" | "revised_julian"
    rule: str = "orthodox"   # which fasting discipline
    menaia: tuple = ()       # extra commemoration sets to overlay
    note: str = ""

    @property
    def calendar(self):
        """The short name people actually use for this combination."""
        if self.paschalion == "gregorian" and self.menologion != "julian":
            return "western"
        return "old" if self.menologion == "julian" else "new"


# The three combinations, for anyone who wants a reckoning without naming a
# church. `--calendar new` is the default and covers most English-speaking
# Orthodox parishes.
CALENDARS = {
    "old": Jurisdiction(
        "old", "Old Calendar", "julian", "julian",
        note="Julian throughout — the reckoning of Moscow, Serbia, Georgia, "
             "Jerusalem and the Holy Mountain."),
    "new": Jurisdiction(
        "new", "New Calendar", "julian", "revised_julian",
        note="Revised Julian fixed feasts with the Julian Paschalion — "
             "Constantinople, Greece, Antioch, Romania, Bulgaria, the OCA."),
    "western": Jurisdiction(
        "western", "Gregorian", "gregorian", "revised_julian",
        note="Gregorian throughout, Pascha included. Kept by the Finnish "
             "Orthodox Church and by most Byzantine Catholics."),
}


_ORTHODOX = [
    # ── old calendar
    ("moscow", "Russian Orthodox Church", "julian", "julian", "orthodox", (), ""),
    ("rocor", "Russian Orthodox Church Outside Russia", "julian", "julian",
     "orthodox", (), ""),
    ("serbian", "Serbian Orthodox Church", "julian", "julian", "orthodox", (), ""),
    ("georgian", "Georgian Orthodox Church", "julian", "julian", "orthodox", (), ""),
    ("jerusalem", "Patriarchate of Jerusalem", "julian", "julian", "orthodox", (), ""),
    ("athos", "The Holy Mountain", "julian", "julian", "orthodox", (),
     "Athonite practice is stricter than the printed Typikon at many points."),
    ("polish", "Polish Orthodox Church", "julian", "julian", "orthodox", (),
     "Returned to the Julian menologion in 2014 after decades on the Revised "
     "Julian; some parishes kept the new calendar."),
    ("macedonian", "Macedonian Orthodox Church", "julian", "julian",
     "orthodox", (), ""),

    # ── new calendar
    ("constantinople", "Ecumenical Patriarchate", "julian", "revised_julian",
     "orthodox", (), ""),
    ("goarch", "Greek Orthodox Archdiocese of America", "julian",
     "revised_julian", "orthodox", (), ""),
    ("greece", "Church of Greece", "julian", "revised_julian", "orthodox", (), ""),
    ("antiochian", "Antiochian Orthodox Church", "julian", "revised_julian",
     "orthodox", (), ""),
    ("oca", "Orthodox Church in America", "julian", "revised_julian",
     "orthodox", (),
     "Most parishes keep the new calendar; a minority, chiefly of Carpatho-"
     "Russian and Alaskan background, keep the old."),
    ("romanian", "Romanian Orthodox Church", "julian", "revised_julian",
     "orthodox", (), ""),
    ("bulgarian", "Bulgarian Orthodox Church", "julian", "revised_julian",
     "orthodox", (), ""),
    ("cyprus", "Church of Cyprus", "julian", "revised_julian", "orthodox", (), ""),
    ("albanian", "Orthodox Church of Albania", "julian", "revised_julian",
     "orthodox", (), ""),
    ("ukrainian", "Orthodox Church of Ukraine", "julian", "revised_julian",
     "orthodox", (),
     "Moved the fixed feasts to the Revised Julian calendar on 1 September "
     "2023. The Julian Paschalion is retained."),

    # ── gregorian throughout
    ("finnish", "Finnish Orthodox Church", "gregorian", "revised_julian",
     "orthodox", (),
     "The only Orthodox Church keeping the Gregorian Paschalion."),
]

_EASTERN_CATHOLIC = [
    ("ruthenian", "Byzantine Catholic Metropolia of Pittsburgh", "gregorian",
     "revised_julian", "ruthenian", ("ruthenian",),
     "Gregorian throughout. Fasting is reported under Ruthenian particular "
     "law, which binds far less than the Typikon prescribes."),
    ("ugcc", "Ukrainian Greek Catholic Church", "julian", "revised_julian",
     "ruthenian", ("ruthenian",),
     "Moved the fixed feasts to the Revised Julian calendar on 1 September "
     "2023, keeping the Julian Paschalion. Eparchies outside Ukraine vary; "
     "several keep the Gregorian Paschalion."),
    ("melkite", "Melkite Greek Catholic Church", "gregorian", "revised_julian",
     "ruthenian", (),
     "Practice is not uniform: parishes in the Middle East commonly keep the "
     "Julian Paschalion for the sake of the local Orthodox."),
    ("romanian-catholic", "Romanian Greek Catholic Church", "gregorian",
     "revised_julian", "ruthenian", (), ""),
]

JURISDICTIONS = {}
for _k, _n, _p, _m, _r, _mn, _note in _ORTHODOX + _EASTERN_CATHOLIC:
    JURISDICTIONS[_k] = Jurisdiction(_k, _n, _p, _m, _r, _mn, _note)


def resolve(name):
    """A jurisdiction key, a calendar name, or None for the default."""
    if name is None:
        return CALENDARS["new"]
    key = name.strip().lower()
    if key in JURISDICTIONS:
        return JURISDICTIONS[key]
    if key in CALENDARS:
        return CALENDARS[key]
    raise KeyError(name)


def names():
    """Everything ``resolve`` accepts, calendars first."""
    return list(CALENDARS) + list(JURISDICTIONS)
