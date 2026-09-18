"""The command line."""

import argparse
import datetime as dt
import shutil
import sys

from . import config
from .day import describe, week, year_of
from .export import to_ics, to_json
from .fasting import OBLIGATORY
from .jurisdictions import CALENDARS, JURISDICTIONS, resolve
from .rules import get as get_rule

__version__ = "0.1.0"


def _list_jurisdictions():
    print("\n  Calendars — a reckoning without naming a church:\n")
    for j in CALENDARS.values():
        print(f"    {j.key:20} {j.name}")
        for line in _wrap(j.note, 56):
            print(f"    {'':20} {line}")
    print("\n  Jurisdictions:\n")
    width = max(len(k) for k in JURISDICTIONS)
    for j in JURISDICTIONS.values():
        cal = {"old": "old calendar", "new": "new calendar",
               "western": "gregorian"}[j.calendar]
        print(f"    {j.key:{width}}  {j.name}")
        print(f"    {'':{width}}  {cal}; {j.rule} fasting rule")
        if j.note:
            for line in _wrap(j.note, 66):
                print(f"    {'':{width}}  {line}")
        print()


def _elide(text, width):
    """Cut on a word boundary and say so, rather than stopping mid-word."""
    if len(text) <= width:
        return text
    cut = text[:width - 1]
    if " " in cut[width // 2:]:
        cut = cut[:cut.rstrip().rfind(" ")]
    return cut.rstrip(" ,;") + "\u2026"


def _wrap(text, width):
    words, line, out = text.split(), "", []
    for w in words:
        if len(line) + len(w) + 1 > width:
            out.append(line)
            line = w
        else:
            line = f"{line} {w}".strip()
    if line:
        out.append(line)
    return out


def _print_days(days, jur, rule, source, verbose):
    if len(days) == 1:
        d = days[0]
        head = f"  {d.weekday} {d.date.strftime('%-d %B %Y')}"
        if d.church_date != d.date.strftime("%-d %B"):
            head += f"   — {d.church_date} in the church calendar"
        print(f"\n{head}")
        print(f"  {jur.name} · {rule['title']}"
              + (f"   (from {source})" if source != "flag" else ""))
        print()
        if d.feast:
            print(f"  {d.emoji}  {d.feast}")
        else:
            print(f"  {d.emoji}  {d.title}")
        if d.cycle:
            print(f"      {d.cycle}")
        print()
        mark = "  — obligatory" if d.obligation == OBLIGATORY else ""
        print(f"      {d.fast_label}{mark}")
        if d.reason:
            print(f"      {d.reason}")
        if d.season:
            print(f"      {d.season}")
        rest = [c for c in d.commemorations if c != d.title]
        if rest:
            print()
            for c in rest:
                print(f"      · {c}")
        if verbose:
            print()
            print(f"      Pascha {d.pascha}  ·  {d.days_to_pascha} days")
        print()
        return

    print(f"\n  {jur.name} · {rule['title']}\n")
    # The glyph is two columns wide in a terminal and one character to Python,
    # which is why the arithmetic below looks off by one.
    fixed = 2 + 2 + 10 + 1 + 1 + 32
    room = max(24, shutil.get_terminal_size((100, 24)).columns - fixed - 1)
    for d in days:
        mark = "*" if d.obligation == OBLIGATORY else " "
        when = d.date.strftime("%a %-d %b")
        print(f"  {d.emoji} {when:>10}{mark} {d.fast_label:<32}{_elide(d.title, room)}")
    if any(d.obligation == OBLIGATORY for d in days):
        print("\n  * obligatory. Everything unmarked is customary and does not bind.")
    print()


def main(argv=None):
    p = argparse.ArgumentParser(
        prog="typikon",
        description="The Orthodox liturgical day — feasts, seasons and the fast.")
    p.add_argument("-j", "--jurisdiction", "-c", "--calendar", dest="jurisdiction",
                   help="a church or a reckoning; see --list")
    p.add_argument("-d", "--date", help="YYYY-MM-DD (default: today)")
    p.add_argument("-n", "--days", type=int, default=1,
                   help="how many days from the start date, inclusive")
    p.add_argument("--json", action="store_true", help="machine-readable output")
    p.add_argument("--ics", action="store_true", help="an iCalendar feed")
    p.add_argument("--indent", type=int, default=None, help="indent JSON output")
    p.add_argument("--year", type=int, help="print a year's key dates and exit")
    p.add_argument("--list", action="store_true",
                   help="list every jurisdiction and calendar")
    p.add_argument("--set-default", metavar="JURISDICTION",
                   help="remember a jurisdiction and exit")
    p.add_argument("--update-menaion", metavar="NAME",
                   help="fetch a supplementary commemoration set (see --list-menaia)")
    p.add_argument("--list-menaia", action="store_true",
                   help="list the supplementary sets that can be fetched")
    p.add_argument("-v", "--verbose", action="store_true")
    p.add_argument("--version", action="version", version=f"typikon {__version__}")
    args = p.parse_args(argv)

    if args.list:
        _list_jurisdictions()
        return 0

    if args.list_menaia:
        from .menaia import SOURCES, target_dir
        print("\n  Supplementary commemorations, fetched on request:\n")
        for key, src in SOURCES.items():
            print(f"    {key:14} {src['title']}")
            print(f"    {'':14} {src['url']}")
            for line in _wrap(src["note"], 62):
                print(f"    {'':14} {line}")
            print()
        print(f"  They are written to {target_dir()}\n")
        return 0

    if args.update_menaion:
        from .menaia import update
        try:
            result = update(args.update_menaion)
        except KeyError as exc:
            print(f"typikon: {exc.args[0]}", file=sys.stderr)
            return 2
        except Exception as exc:
            print(f"typikon: could not fetch: {exc}", file=sys.stderr)
            return 1
        print(f"  {result['days']} days written to {result['path']}")
        print(f"  Source: {result['source']['title']}")
        if result["corrections"]:
            print(f"  source typos corrected: {', '.join(result['corrections'])}")
        if result["missing"]:
            print(f"  MISSING: {', '.join(result['missing'])}")
            return 1
        return 0

    if args.set_default:
        try:
            jur = resolve(args.set_default)
        except KeyError:
            print(f"typikon: unknown jurisdiction {args.set_default!r};"
                  " try --list", file=sys.stderr)
            return 2
        where = config.write(jur.key)
        print(f"  {jur.name} — written to {where}")
        return 0

    if args.jurisdiction:
        name, source = args.jurisdiction, "flag"
    else:
        name, source = config.default_jurisdiction()

    try:
        jur = resolve(name)
    except KeyError:
        print(f"typikon: unknown jurisdiction {name!r}; try --list", file=sys.stderr)
        return 2

    if args.year is not None:
        if not 1 <= args.year <= 9999:
            print(f"typikon: {args.year} is not a year this calendar reaches",
                  file=sys.stderr)
            return 2
        mv = year_of(args.year, jur)
        print(f"\n  {args.year} · {jur.name}\n")
        for k in ("publican_pharisee", "meatfare", "cheesefare", "clean_monday",
                  "great_fast_end", "lazarus_saturday", "palm_sunday",
                  "great_friday", "pascha", "ascension", "pentecost",
                  "all_saints", "apostles_start", "apostles_end",
                  "dormition", "nativity", "theophany"):
            d = mv[k]
            print(f"    {k.replace('_', ' '):20} {d.isoformat()}  {d.strftime('%a')}")
        print()
        return 0

    if args.date:
        try:
            start = dt.date.fromisoformat(args.date)
        except ValueError:
            print(f"typikon: {args.date!r} is not a date \u2014 expected YYYY-MM-DD",
                  file=sys.stderr)
            return 2
    else:
        start = dt.date.today()
    days = week(start, args.days, jur)

    if args.json:
        print(to_json(days, args.indent))
        return 0
    if args.ics:
        sys.stdout.write(to_ics(days, name=f"{jur.name} — Typikon"))
        return 0

    _print_days(days, jur, get_rule(jur.rule), source, args.verbose)

    # The first run should say what it assumed, and then stop saying it.
    if source == "default":
        where = config.write(jur.key)
        print(f"  Reckoning by the {jur.name.lower()}, written to {where}.")
        print("  Change it with `typikon --set-default <jurisdiction>`,"
              " or see `typikon --list`.\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
