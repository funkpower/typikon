# typikon

The Orthodox liturgical day: feasts, seasons and the fast, computed rather
than fetched. The Byzantine Catholic churches are here too, on their own
calendars and, where the law has been sourced, under their own particular law.

Everything derives from the paschalion and the fixed calendar, so it needs no
network, cannot go stale, and answers for any year you ask about. No API key,
no service that might be gone next year, no cache to invalidate.

```console
$ typikon --jurisdiction rocor --date 2026-03-18

  Wednesday 18 March 2026   — 5 March in the church calendar
  Russian Orthodox Church Outside Russia · the Typikon · slavic usage

  🥬  The Holy Martyr Conon

      xerophagy — no oil or wine
      a weekday of the Great Fast
      Great Fast
```

A week at a time, which is what you want the day before a fast begins:

```console
$ typikon --jurisdiction antiochian --date 2027-04-26 --days 7

  Antiochian Orthodox Church · the Typikon

  🥬 Mon 26 Apr  xerophagy — no oil or wine      The Holy Hieromartyr Basil, Bishop…
  🥬 Tue 27 Apr  xerophagy — no oil or wine      The Holy Hieromartyr Symeon, kinsman…
  🥬 Wed 28 Apr  xerophagy — no oil or wine      The Holy Apostles Jason and Sosipater
  🫒 Thu 29 Apr  wine and oil permitted          Great and Holy Thursday
  ⛔ Fri 30 Apr  no food until evening           Great and Holy Friday
  🥬  Sat 1 May  xerophagy — no oil or wine      Great and Holy Saturday
  🍖  Sun 2 May  no fast                         The Radiant and Life-giving…
```

## Install

One static binary, no runtime to install first.

```console
$ cargo install --git https://git.pumpin.online/funkpower/typikon
```

Rust 1.85 or newer. The name `typikon` on crates.io belongs to an unrelated
project, so `cargo install typikon` and a `typikon = "1"` dependency will not
get you this; install and depend on it by git URL.

Or build it from a checkout:

```console
$ git clone https://git.pumpin.online/funkpower/typikon
$ cd typikon && cargo build --release
$ install -Dm755 target/release/typikon ~/.local/bin/typikon
```

The menologion is compiled in, so the binary answers for every day of the year
on its own: no data file to find, no network, nothing to go stale.

For Python, there is a wheel:

```console
$ pip install typikon
```

The menologion is compiled into that too, so it carries no data files and needs
no network either. Wheels are built for Linux on x86-64 and aarch64; everywhere
else pip falls back to the source distribution, which needs a Rust toolchain of
1.85 or newer on the installing machine.

## Which calendar?

Three things get conflated constantly, and keeping them apart is most of what
this library is for:

| | |
|---|---|
| **Paschalion** | which table gives Pascha, Julian or Gregorian |
| **Menologion** | which calendar the fixed feasts are read on |
| **Rule** | which fasting discipline is kept |

Almost every Orthodox Church keeps the **Julian Paschalion**. What divides them
is the menologion. The old-calendar churches read the fixed feasts on the
Julian calendar too; the new-calendar churches read them on the Revised Julian,
which agrees with the Gregorian for every date until 2800. So a new-calendar
church keeps the Nativity with the West and Pascha with the East, not a
compromise but exactly what the 1923 congress decided.

### Greek or Slavic usage

Both keep the same Typikon and read it a little differently. Two of those
differences are modelled, and each is visible in the output:

| | Slavic | Greek |
|---|---|---|
| The Protection, 1 October | great feast | lower rank; Greece keeps it on 28 October besides |
| Fish in the Nativity Fast | Saturdays and Sundays | any day but Wednesday and Friday, until 17 December |

Every jurisdiction carries the usage its books follow. `--tradition slavic` or
`--tradition greek` overrides it, which is what you want for a parish that does
not match its jurisdiction's default.

```console
$ typikon -j goarch --date 2026-10-01 --format tsv --fields rank,sign,fast,title
rank	sign	fast	title
vigil	✚	none	The Protection of our Most Holy Lady the Theotokos
```

```console
$ typikon -j goarch --date 2026-10-01 --tradition slavic --format tsv --fields rank,sign,fast,title
rank	sign	fast	title
great	⊕	none	The Protection of our Most Holy Lady the Theotokos
```

The rank is not decoration: it is what lifts a Wednesday or Friday fast, so the
usage changes what you eat as well as what you sing.

### Which calendar?

Pick a reckoning without naming a church:

```console
$ typikon --calendar old        # Julian throughout
$ typikon --calendar new        # Revised Julian fixed feasts, Julian Pascha
$ typikon --calendar western    # Gregorian throughout
```

Or name one, which also picks up its fasting rule and any local saints:

```console
$ typikon --jurisdiction rocor
$ typikon --jurisdiction antiochian --days 7
$ typikon --list                # every accepted value
```

The first run writes your choice to `~/.config/typikon/config.toml` and stops
asking. `--set-default` remembers the whole invocation, not just the
jurisdiction:

```console
$ typikon --set-default ruthenian --typikon
```

records the discipline and the usage alongside the church, so a terminal and a
desktop widget on the same machine cannot end up disagreeing about the same
day. Override any of it for one run with the flags, or set
`TYPIKON_JURISDICTION`.

Twenty Orthodox jurisdictions ship: the old-calendar churches (Moscow, ROCOR,
Serbia, Georgia, Jerusalem, the Holy Mountain, Poland, Macedonia, ACROD), the
new-calendar ones (Constantinople, Greece, GOARCH, Antioch, the OCA, Romania,
Bulgaria, Cyprus, Albania, Ukraine), and the Finnish Orthodox Church, alone
among the Orthodox in keeping the Gregorian Paschalion. Four Byzantine
Catholic churches ship alongside them.

## The Byzantine Catholic churches

Four are modelled, and they are not a footnote on the Orthodox ones: they keep
different paschalions, and their fasting is a different kind of thing.

| | calendar | fasting |
|---|---|---|
| `ruthenian` | Gregorian throughout | Ruthenian particular law, sourced |
| `ugcc` | Revised Julian feasts, Julian Pascha since 2023 | its own; not modelled |
| `melkite` | Gregorian; Middle Eastern parishes often keep Julian Pascha | its own; not modelled |
| `romanian-catholic` | Gregorian throughout | its own; not modelled |

**A church's law is never borrowed from another.** Ruthenian particular law is
sourced from the Metropolia of Pittsburgh and is reported for that church only.
The rest are shown at the Typikon's discipline with **nothing reported as
binding**, because their own law has not been sourced and inventing one would
tell a Melkite that fourteen days a year oblige him under a law that was never
his. When a church's law is added, its entry will say so and its obligations
will appear. A test holds that line.

See [the law and the fast](#the-law-and-the-fast-are-not-the-same-question)
for what `--typikon` does with this, which is the flag most Byzantine Catholics
will want.

Ten of the fourteen Byzantine-rite Eastern Catholic churches have no entry of
their own. Each keeps one of the three reckonings, and
[docs/calendars.md](https://git.pumpin.online/funkpower/typikon/src/branch/main/docs/calendars.md) sets out which questions decide it.

## The fast

Six levels, ordered from strictest to laxest. They are the Typikon's own
distinctions, not a simplification of them:

| | |
|---|---|
| `strict` | no food until evening |
| `xerophagy` | no oil, no wine; no meat, fish, eggs or dairy |
| `wine_oil` | wine and oil permitted |
| `fish` | fish, wine and oil permitted |
| `dairy` | dairy, eggs and fish permitted; no meat |
| `none` | no fast |

A calendar that reports xerophagy and wine-and-oil both as "fasting" has thrown
away the information its reader wanted.

**Level and obligation are separate questions.** The Orthodox Church legislates
fasting as tradition rather than as precept, so every Orthodox day here reports
as customary. The Eastern Catholic churches do have law about it, and it
requires very much less than the Typikon describes, so those jurisdictions
report both the level at which the day is kept and the minimum that actually
binds. A calendar saying "obligatory" where nothing obliges is worse than no
calendar; one reporting only the binding minimum makes the obligatory days look
laxer than the voluntary ones beside them.

### The law and the fast are not the same question

For an Eastern Catholic church the two come apart sharply, and reporting only
the law makes the Great Fast look like four days of abstinence a year. That is
what binds; it is not what is kept.

What the law requires, which is the first four days of the Great Fast:

```console
$ typikon -j ruthenian --no-typikon --days 4 --date 2026-02-16

  Byzantine Catholic Metropolia of Pittsburgh · Ruthenian particular law

  🥬 Mon 16 Feb* no meat, fish, eggs or dairy    Clean Monday — the first day of the…
  🥬 Tue 17 Feb  no meat, fish, eggs or dairy    The Holy Great Martyr Theodore the…
  🥬 Wed 18 Feb* no meat, fish, eggs or dairy    Our Holy Father Leo the Great, Pope…
  🥬 Thu 19 Feb  no meat, fish, eggs or dairy    The Holy Apostles Archippus and…

  * obligatory. Everything unmarked is customary and does not bind.
```

And what is kept, with `--typikon`:

```console
$ typikon -j ruthenian --typikon --days 4 --date 2026-02-16

  Byzantine Catholic Metropolia of Pittsburgh · the Typikon, obliging under Ruthenian particular law

  ⛔ Mon 16 Feb* no food until evening           Clean Monday — the first day of the…
  🫒 Tue 17 Feb  wine and oil permitted          The Holy Great Martyr Theodore the…
  🥬 Wed 18 Feb* xerophagy — no oil or wine      Our Holy Father Leo the Great, Pope…
  🥬 Thu 19 Feb  xerophagy — no oil or wine      The Holy Apostles Archippus and…

  * obligatory. Everything unmarked is customary and does not bind.
```

The same three days are starred in both. That is the point: the flag changes
what is *kept*, not what *binds*. Tuesday relaxes to wine and oil because St
Theodore the Recruit is of polyeleos rank, which is the menologion doing its
work.

`--no-typikon` forces the law back, for when the config remembers otherwise.
Either way it is a no-op for a jurisdiction already reckoning by the Typikon.

Where usages differ (Great Thursday, the Exaltation, Lazarus Saturday), this
follows the common published rule and names it in the day's reason so you can
disagree with it. Monastic practice, and the Holy Mountain in particular, is
stricter than all of it. A calendar cannot settle that; your bishop and your
spiritual father can.

## Output

```console
$ typikon --days 7                       # for a person
$ typikon --format json --days 7         # for a bar, a widget, a script
$ typikon --format ics --days 365 > orthodox.ics
```

Delimited output, for a spreadsheet or a pipeline. RFC 4180 quoting, one header
row, one row per day:

```console
$ typikon -j rocor --date 2026-03-16 --days 3 --format csv \
      --fields date,church_date,fast,title

date,church_date,fast,title
2026-03-16,3 March,xerophagy,"The Holy Martyrs Eutropius, Cleonicus and Basiliscus"
2026-03-17,4 March,xerophagy,Our Venerable Father Gerasimus of the Jordan
2026-03-18,5 March,xerophagy,The Holy Martyr Conon
```

A year's key dates, to check against the one your parish prints:

```console
$ typikon --year 2027 -j goarch

  2027 · Greek Orthodox Archdiocese of America

    publican pharisee    2027-02-21  Sun
    meatfare             2027-03-07  Sun
    cheesefare           2027-03-14  Sun
    clean monday         2027-03-15  Mon
    great fast end       2027-04-23  Fri
    lazarus saturday     2027-04-24  Sat
    palm sunday          2027-04-25  Sun
    great friday         2027-04-30  Fri
    pascha               2027-05-02  Sun
    …
```

Every day of vigil rank or above carries the mark its books print beside it:
⊕ for a great feast, ✚ for a vigil, ☩ for a polyeleos. It is the `sign`
field in the JSON and a column in the delimited formats.

`--no-emoji` drops the glyphs from every format, for a terminal or a bar
without them. `--list-fields` prints the columns `csv` and `tsv` can take, and
`--fields` picks a subset in the order you name them. `--json` and `--ics`
still work as shorthand.

The JSON shape is flat and stable. Front ends are the thing most likely to be
written by someone else, and a schema that shifts underneath them is a bad
neighbour.

```console
$ typikon -j rocor --date 2026-03-18 --json --indent 2
{
  "date": "2026-03-18",
  "weekday": "Wednesday",
  "jurisdiction": "rocor",
  "calendar": "old",
  "churchDate": "5 March",
  "season": "Great Fast",
  "cycle": null,
  "feast": null,
  "rank": null,
  "sign": null,
  "tradition": "slavic",
  "title": "The Holy Martyr Conon",
  "commemorations": [
    "The Holy Martyr Conon"
  ],
  "fast": "xerophagy",
  "fastLabel": "xerophagy — no oil or wine",
  "emoji": "🥬",
  "reason": "a weekday of the Great Fast",
  "obligation": "customary",
  "binds": null,
  "pascha": "2026-04-12",
  "daysToPascha": 25,
  "julianOffset": 13
}
```

One day is an object; several are `{"days": [...]}`.

## As a library

```rust
use typikon::calendars::Date;
use typikon::day::describe;
use typikon::jurisdictions::resolve;

let rocor = resolve(Some("rocor"))?;
let day = describe(Date::from_ymd(2026, 9, 18).unwrap(), rocor);

day.church_date;     // "5 September"
day.fast;            // Level::Xerophagy
day.title();         // "The Holy Prophet Zachariah, father of the Forerunner"
day.is_obligatory(); // false

typikon::paschalion::julian_pascha(2027);     // 2027-05-02
typikon::paschalion::gregorian_pascha(2027);  // 2027-03-28
```

The levels are an ordered enum, strictest first, so `a < b` reads as "a is
stricter than b" and a feast can only ever relax a fast:

```rust
Level::Xerophagy.relaxed_to(Level::Fish, Level::WineOil)  // WineOil: no saint
                                                          // brings fish into Lent
Level::Fish.relaxed_to(Level::WineOil, Level::WineOil)    // Fish: never tightens
```

## From Python

```python
import datetime, typikon

day = typikon.describe(datetime.date(2026, 9, 18))

day.church_date     # '5 September'
day.fast            # 'xerophagy'
day.title           # 'The Holy Prophet Zachariah, father of the Forerunner'
day.is_obligatory() # False

typikon.today()
typikon.pascha(2027)                                  # datetime.date(2027, 5, 2)
typikon.week(datetime.date(2026, 4, 12), 7)           # seven consecutive days
typikon.describe(d, jurisdiction='rocor')
[j['key'] for j in typikon.jurisdictions()]
```

Dates in and out are `datetime.date`. `to_dict()` gives exactly the fields
`--json` gives, so anything already reading the CLI's output can read a `Day`
without changing shape — there is a test that holds the two to that.

Given no `jurisdiction=`, the library reads the same configured default the
command line does. Calling the engine's own resolver instead would have quietly
answered for a different church than `typikon --set-default` selected, which is
the kind of disagreement nobody notices until a fast day is wrong.

The wheel is abi3, so one build serves every Python from 3.9 up. It ships
`py.typed` and a stub file, and `mypy --strict` is clean against it.

## From C, and anything with an FFI

```sh
cargo build --release --features capi   # target/release/libtypikon.so
cc -Iinclude yours.c -ltypikon
```

```c
#include <typikon.h>

typikon_day *d = typikon_describe(typikon_today(), "rocor");
if (!d) { fputs(typikon_last_error(), stderr); return 1; }

typikon_day_church_date(d);    /* "5 September" */
typikon_day_fast(d);           /* "xerophagy" */
typikon_day_emoji(d);          /* the glyph for the level */
typikon_day_title(d);          /* the feast, else the principal commemoration */
typikon_day_is_obligatory(d);  /* 0 */

typikon_day_free(d);
```

Dates cross the boundary as **Julian Day Numbers**, so no struct layout is part
of the ABI and there is nothing for a caller to keep in step;
`typikon_jdn_from_ymd` converts. A day is opaque and owns its strings: an
accessor returns the same borrowed pointer every time, so reading one in a
render loop allocates nothing, and nothing it returns should be freed. NULL
from an accessor means the day has no such field, which is not an error. A call
that can fail returns NULL or 0 and leaves a message in `typikon_last_error()`.

[`include/typikon.h`](https://git.pumpin.online/funkpower/typikon/src/branch/main/include/typikon.h)
is the whole surface: it carries every field of the JSON, so a statusline does
not need a parser. [`examples/c/demo.c`](https://git.pumpin.online/funkpower/typikon/src/branch/main/examples/c/demo.c)
is a working one.

### From Lua

[`examples/lua/typikon.lua`](https://git.pumpin.online/funkpower/typikon/src/branch/main/examples/lua/typikon.lua)
is a LuaJIT FFI binding, for AwesomeWM, WezTerm, Neovim and anything else that
embeds LuaJIT and should not be spawning a process on every redraw.

```lua
local typikon = require("typikon")

local day = assert(typikon.today({ jurisdiction = "rocor" }))
day.church_date   -- "5 September"
day.fast          -- "xerophagy"
day.title         -- "The Holy Prophet Zachariah, father of the Forerunner"
day.commemorations
```

## The menologion

The fixed calendar ships as data and is indexed by **ecclesiastical** date, so
one file serves every jurisdiction. An old-calendar church and a new-calendar
church both keep the Nativity on 25 December; they disagree about which morning
that is, and that disagreement lives in the calendar conversion, not in the
data.

`data/menologion.json`, compiled into the binary, covers all 366 days with the principal
commemorations of the common Byzantine tradition, ranked so the fasting rules
know when a feast lifts a Wednesday. It was composed for this project and is
under the same licence as the code. It is a working set, not a complete
Synaxarion; additions are welcome.

Some churches also keep their own saints, and those lists are usually published
by a body that holds the copyright in the wording. Those are **not** shipped.
Fetch one onto your own machine instead:

```console
$ typikon --list-menaia
$ typikon --update-menaion ruthenian
```

It lands in `~/.local/share/typikon/` and is overlaid automatically.

## Further reading

- [docs/calendars.md](https://git.pumpin.online/funkpower/typikon/src/branch/main/docs/calendars.md) sets out which churches keep which
  reckoning, how the split came about, what is modelled here and what is not.
- [docs/versioning.md](https://git.pumpin.online/funkpower/typikon/src/branch/main/docs/versioning.md) says what counts as the public
  surface, and how a correction to the calendar itself is announced.
- [CHANGELOG.md](https://git.pumpin.online/funkpower/typikon/src/branch/main/CHANGELOG.md) carries those announcements.

## Accuracy

`typikon` is pinned against published calendars: the paschalion for eleven
named years and its invariants across the two centuries from 1900, 146,000
calendar round-trips from 1800 to 2200, the fast on every day that is not open
to interpretation, and every jurisdiction over the twenty-one years from 2020.
Every title the program can print is cut at every width it can be asked to
print at, because that is where it once crashed.

It is a calendar, not an authority. If it disagrees with the one your parish
prints, your parish is right. Please open an issue.

## Front ends

[`menologion`](https://git.pumpin.online/funkpower/menologion): desktop
widgets for Hyprland (Quickshell), waybar and eww, all reading the JSON above.

![The menologion widget](https://git.pumpin.online/funkpower/menologion/raw/branch/main/docs/nativity-fast.png)

## Releasing

`main` is always releasable; a `vX.Y.Z` tag on it builds the wheels and publishes
to PyPI. [docs/releasing.md](https://git.pumpin.online/funkpower/typikon/src/branch/main/docs/releasing.md) has the workflow, and what has to
be set up before the first one.

## Licence

GPL-2.0-only. The source for any published release is the source distribution
beside it on PyPI, and `Cargo.lock` pins every dependency to an exact version.
See [LICENSE](https://git.pumpin.online/funkpower/typikon/src/branch/main/LICENSE). The menologion is under it too;
`data/LICENSE` says so and notes what the fetched sets are not.
