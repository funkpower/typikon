"""iCalendar, so the calendar can live where the rest of your year does.

Each day becomes one all-day event. The summary is the feast or the principal
commemoration; the description carries the fast, the season and the rest of the
day's commemorations. Days with nothing to say — no feast, no fast — are left
out, because a subscribed calendar that marks all 365 days is one nobody keeps.
"""

import datetime as dt

__all__ = ["to_ics"]

_FOLD = 73


_BACKSLASH = chr(92)

# Order matters: the backslash has to be doubled before anything else
# introduces one, or the escapes escape each other.
_ESCAPES = (
    (_BACKSLASH, _BACKSLASH * 2),
    (";", _BACKSLASH + ";"),
    (",", _BACKSLASH + ","),
    ("\n", _BACKSLASH + "n"),
)


def _esc(text):
    out = str(text)
    for char, replacement in _ESCAPES:
        out = out.replace(char, replacement)
    return out


def _fold(line):
    """RFC 5545 wants lines under 75 octets, continued with a leading space."""
    out = [line[:_FOLD]]
    rest = line[_FOLD:]
    while rest:
        out.append(" " + rest[:_FOLD - 1])
        rest = rest[_FOLD - 1:]
    return "\r\n".join(out)


def to_ics(days, name="Typikon", stamp=None):
    stamp = (stamp or dt.datetime.now(dt.timezone.utc)).strftime("%Y%m%dT%H%M%SZ")
    lines = [
        "BEGIN:VCALENDAR",
        "VERSION:2.0",
        "PRODID:-//typikon//EN",
        "CALSCALE:GREGORIAN",
        "METHOD:PUBLISH",
        f"X-WR-CALNAME:{_esc(name)}",
    ]

    for d in days:
        if not d.feast and d.fast == "none":
            continue
        start = d.date.strftime("%Y%m%d")
        end = (d.date + dt.timedelta(1)).strftime("%Y%m%d")
        summary = d.title or d.fast_label
        if d.emoji:
            summary = f"{d.emoji} {summary}"

        body = []
        if d.fast != "none":
            body.append(f"Fast: {d.fast_label}"
                        + (f" ({d.reason})" if d.reason else ""))
        if d.obligation == "obligatory":
            body.append("Obligatory.")
        if d.season:
            body.append(f"Season: {d.season}")
        if d.cycle:
            body.append(d.cycle[0].upper() + d.cycle[1:])
        body.append(f"Church date: {d.church_date}")
        for c in d.commemorations:
            body.append(c)

        uid = f"{start}-{d.jurisdiction}@typikon"
        lines += [
            "BEGIN:VEVENT",
            f"UID:{uid}",
            f"DTSTAMP:{stamp}",
            f"DTSTART;VALUE=DATE:{start}",
            f"DTEND;VALUE=DATE:{end}",
            _fold(f"SUMMARY:{_esc(summary)}"),
            _fold(f"DESCRIPTION:{_esc(chr(10).join(body))}"),
            "TRANSP:TRANSPARENT",
            "END:VEVENT",
        ]

    lines.append("END:VCALENDAR")
    return "\r\n".join(lines) + "\r\n"
