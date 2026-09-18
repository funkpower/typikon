"""The fixed calendar, and the movable feasts that outrank it.

The menologion ships once and serves every jurisdiction, because it is indexed
by **ecclesiastical** date. A church on the old calendar and a church on the new
both keep the Nativity on 25 December; they disagree about which morning that
is, and that disagreement lives in the calendar conversion, not here.

Supplementary sets — a local church's own saints — overlay the common one
rather than replacing it, so a jurisdiction can add without forking the file.
"""

import json
import os

__all__ = ["Menologion", "load", "MOVABLE_FEASTS", "movable_feast_for"]

_DATA = os.path.join(os.path.dirname(os.path.abspath(__file__)), "data")
_cache = {}


class Menologion:
    def __init__(self, entries):
        self.entries = entries

    def get(self, month, day):
        return self.entries.get(f"{month:02d}-{day:02d}", {})

    def overlay(self, other):
        """Add another set on top of this one, keeping both days' entries."""
        merged = {k: dict(v) for k, v in self.entries.items()}
        for key, add in other.entries.items():
            if key not in merged:
                merged[key] = dict(add)
                continue
            base = merged[key]
            base["commemorations"] = (list(base.get("commemorations", []))
                                      + list(add.get("commemorations", [])))
            # A supplementary set may raise a day's rank but never lower it.
            for field in ("rank", "cycle"):
                if add.get(field):
                    base[field] = add[field]
        return Menologion(merged)


def load(name="menologion"):
    """Load a menologion by name from the packaged data directory."""
    if name in _cache:
        return _cache[name]
    path = os.path.join(_DATA, f"{name}.json")
    if not os.path.exists(path):
        raise FileNotFoundError(path)
    with open(path, encoding="utf-8") as fh:
        m = Menologion(json.load(fh))
    _cache[name] = m
    return m


def for_jurisdiction(jur):
    """The common menologion with any supplementary sets overlaid."""
    m = load()
    for extra in jur.menaia:
        try:
            m = m.overlay(load(extra))
        except FileNotFoundError:
            # A supplementary set is optional; its absence is not an error.
            pass
    return m


# ── the movable feasts, named by their offset from Pascha
#
# These outrank whatever the fixed calendar has for the day. The keys are
# entries in the movable cycle; the value is (title, rank).
MOVABLE_FEASTS = {
    "pascha":             ("The Radiant and Life-giving Resurrection of our Lord — Holy Pascha", "great"),
    "palm_sunday":        ("The Entry of our Lord into Jerusalem", "great"),
    "ascension":          ("The Ascension of our Lord, God and Saviour Jesus Christ", "great"),
    "pentecost":          ("Holy Pentecost — the Descent of the Holy Spirit", "great"),
    "lazarus_saturday":   ("Lazarus Saturday", "vigil"),
    "great_thursday":     ("Great and Holy Thursday", "vigil"),
    "great_friday":       ("Great and Holy Friday", "vigil"),
    "holy_saturday":      ("Great and Holy Saturday", "vigil"),
    "thomas_sunday":      ("Thomas Sunday — the Antipascha", "polyeleos"),
    "myrrhbearers":       ("The Sunday of the Holy Myrrh-bearing Women", "polyeleos"),
    "midfeast":           ("The Midfeast of Pentecost", "polyeleos"),
    "all_saints":         ("The Sunday of All Saints", "polyeleos"),
    "publican_pharisee":  ("The Sunday of the Publican and the Pharisee", None),
    "prodigal_son":       ("The Sunday of the Prodigal Son", None),
    "meatfare":           ("Meatfare Sunday — the Sunday of the Last Judgement", None),
    "cheesefare":         ("Cheesefare Sunday — the Expulsion of Adam, Forgiveness Sunday", None),
    "orthodoxy_sunday":   ("The Sunday of Orthodoxy", "polyeleos"),
    "gregory_palamas":    ("The Sunday of St Gregory Palamas", None),
    "cross_veneration":   ("The Sunday of the Veneration of the Cross", "polyeleos"),
    "john_climacus":      ("The Sunday of St John Climacus", None),
    "mary_of_egypt":      ("The Sunday of St Mary of Egypt", None),
    "fathers_first_council": ("The Sunday of the Fathers of the First Council", None),
    "clean_monday":       ("Clean Monday — the first day of the Great Fast", None),
}


def movable_feast_for(civil, mv):
    """(title, rank) if the day is a movable feast, else (None, None)."""
    for key, (title, rank) in MOVABLE_FEASTS.items():
        if mv.get(key) == civil:
            return title, rank
    return None, None
