# typikon

The Orthodox liturgical day: feasts, seasons and the fast, computed rather
than fetched.

Everything derives from the paschalion and the fixed calendar, so it needs no
network, cannot go stale, and answers for any year you ask about. No API key,
no service that might be gone next year, no cache to invalidate.

![typikon on the command line](docs/cli.png)

```console
$ typikon
  Wednesday 18 March 2026   — 5 March in the church calendar
  Russian Orthodox Church Outside Russia · the Typikon

  🥬  The Holy Martyr Conon

      xerophagy — no oil or wine
      a weekday of the Great Fast
      Great Fast
```

## Install

One static binary, no runtime to install first.

```console
$ cargo install --git https://git.pumpin.online/funkpower/typikon
```

Or build it from a checkout:

```console
$ git clone https://git.pumpin.online/funkpower/typikon
$ cd typikon && cargo build --release
$ install -Dm755 target/release/typikon ~/.local/bin/typikon
```

The menologion is compiled in, so the binary answers for every day of the year
on its own: no data file to find, no network, nothing to go stale.

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
asking. Change it with `--set-default`, override it for one run with the flag,
or set `TYPIKON_JURISDICTION`.

Twenty-three jurisdictions ship: the old-calendar churches (Moscow, ROCOR,
Serbia, Georgia, Jerusalem, the Holy Mountain, Poland, Macedonia), the
new-calendar ones (Constantinople, Greece, GOARCH, Antioch, the OCA, Romania,
Bulgaria, Cyprus, Albania, Ukraine), the Finnish Orthodox Church, alone among
the Orthodox in keeping the Gregorian Paschalion, and, as an add-on, four
Byzantine Catholic churches.

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

Where usages differ (Great Thursday, the Exaltation, Lazarus Saturday), this
follows the common published rule and names it in the day's reason so you can
disagree with it. Monastic practice, and the Holy Mountain in particular, is
stricter than all of it. A calendar cannot settle that; your bishop and your
spiritual father can.

## Output

```console
$ typikon --json --days 7        # for a bar, a widget, a script
$ typikon --ics --days 365 > orthodox.ics
$ typikon --year 2027            # a year's key dates, to check against yours
```

The JSON shape is flat and stable. Front ends are the thing most likely to be
written by someone else, and a schema that shifts underneath them is a bad
neighbour.

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

## Accuracy

`typikon` is pinned against published calendars: the paschalion for six years
by both reckonings and its invariants across two centuries, 146,000 calendar
round-trips from 1800 to 2200, the fast on every day that is not open to
interpretation, and every jurisdiction over twenty years.

It was ported from a Python implementation, and the port was checked by
running both over 199,446 day-records, 26 jurisdictions across 21 years, and
requiring every field of every record to match.

It is a calendar, not an authority. If it disagrees with the one your parish
prints, your parish is right. Please open an issue.

## Front ends

[`menologion`](https://git.pumpin.online/funkpower/menologion): desktop
widgets for Hyprland (Quickshell), waybar and eww, all reading the JSON above.

![The menologion widget](https://git.pumpin.online/funkpower/menologion/raw/branch/main/docs/nativity-fast.png)

## Licence

GPL-2.0-only. See [LICENSE](LICENSE). The menologion is under it too;
`data/LICENSE` says so and notes what the fetched sets are not.
