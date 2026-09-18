"""Supplementary commemorations, fetched rather than shipped.

The menologion this library ships is the common Byzantine one, composed from
the tradition and given away (see data/LICENSE). Some churches also keep their
own saints, and those lists are usually published by a body that holds the
copyright in the wording. Redistributing them would be taking someone else's
work; fetching them onto your own machine, for your own use, is not.

So supplementary sets are **not** in the repository. ``typikon
--update-menaion ruthenian`` fetches one into your data directory, where the
menologion loader picks it up automatically and overlays it on the common set.

Each source declares where it lives and how to read it. Adding one is a dict.
"""

import datetime as dt
import json
import os
import re
import urllib.request

__all__ = ["SOURCES", "update", "target_dir"]

MONTHS = ("January February March April May June July August "
          "September October November December").split()

SOURCES = {
    "ruthenian": {
        "title": "Byzantine Catholic Metropolia of Pittsburgh",
        "url": "https://mci.archpitt.org/liturgy/Calendar_of_Saints.html",
        "note": "Published by the Metropolitan Cantor Institute. Fetched for "
                "your own use; not redistributed with this package.",
    },
}


def target_dir():
    """Where a fetched set is written.

    The package directory if it is writable — a checkout usually is — and the
    user's data directory otherwise, so an installed copy still works.
    """
    packaged = os.path.join(os.path.dirname(os.path.abspath(__file__)), "data")
    if os.access(packaged, os.W_OK):
        return packaged
    base = os.environ.get("XDG_DATA_HOME") or os.path.expanduser("~/.local/share")
    path = os.path.join(base, "typikon")
    os.makedirs(path, exist_ok=True)
    return path


def _strip_html(html):
    txt = re.sub(r"<script.*?</script>|<style.*?</style>", "", html, flags=re.S)
    txt = re.sub(r"<(br|/p|/div|/li|/h\d)[^>]*>", "\n", txt, flags=re.I)
    txt = re.sub(r"<[^>]+>", "\n", txt)
    for a, b in (("&nbsp;", " "), ("&amp;", "&"), ("&#8217;", "'"),
                 ("&rsquo;", "'"), ("&mdash;", "—")):
        txt = txt.replace(a, b)
    return [x for x in (re.sub(r"\s+", " ", l).strip() for l in txt.split("\n")) if x]


def _parse_mci(html):
    """MCI publishes prose, not a table, so entries split on the date headings.

    The page has at least one typo — 14 October is written "October 1" — which
    silently overwrites the real entry for that day, taking the Protection of
    the Theotokos with it. A date that goes backwards inside a month is the
    giveaway, and is corrected to follow the previous one.
    """
    lines = _strip_html(html)
    idx = {m: i + 1 for i, m in enumerate(MONTHS)}
    dated = re.compile(r"^(" + "|".join(MONTHS) + r")\s+(\d{1,2})\s*(.*)$")
    heading = re.compile(r"^(" + "|".join(MONTHS) + r")\.?$")

    out, cur, buf, corrections = {}, None, [], []
    prev_month = prev_day = None

    def flush():
        if not (cur and buf):
            return
        items = []
        for piece in buf:
            for part in re.split(r"(?<=\.)\s+(?=[A-Z(])", piece):
                part = part.strip(" .;")
                if len(part) > 3 and not dated.match(part) and not heading.match(part):
                    items.append(part)
        if items:
            out[cur] = {"commemorations": items}

    for line in lines:
        if heading.match(line):
            continue
        m = dated.match(line)
        if m:
            flush()
            month, day = idx[m.group(1)], int(m.group(2))
            if prev_month == month and day <= prev_day:
                corrections.append(
                    f"{month:02d}-{day:02d} -> {month:02d}-{prev_day + 1:02d}")
                day = prev_day + 1
            prev_month, prev_day = month, day
            cur = f"{month:02d}-{day:02d}"
            buf = [m.group(3)] if m.group(3).strip() else []
        elif cur:
            buf.append(line)
    flush()
    return out, corrections


_PARSERS = {"ruthenian": _parse_mci}


def update(name):
    """Fetch a supplementary set and write it beside the packaged menologion."""
    try:
        source = SOURCES[name]
    except KeyError:
        raise KeyError(f"unknown menaion: {name!r}; "
                       f"try one of {', '.join(SOURCES)}") from None

    req = urllib.request.Request(source["url"],
                                 headers={"User-Agent": "typikon"})
    with urllib.request.urlopen(req, timeout=30) as r:
        html = r.read().decode("utf-8", "replace")

    entries, corrections = _PARSERS[name](html)

    expected = set()
    for month in range(1, 13):
        for day in range(1, 32):
            try:
                dt.date(2024, month, day)
                expected.add(f"{month:02d}-{day:02d}")
            except ValueError:
                pass
    missing = sorted(expected - set(entries))

    path = os.path.join(target_dir(), f"{name}.json")
    with open(path, "w", encoding="utf-8") as fh:
        json.dump(entries, fh, ensure_ascii=False, indent=1, sort_keys=True)

    return {"path": path, "days": len(entries), "corrections": corrections,
            "missing": missing, "source": source}
