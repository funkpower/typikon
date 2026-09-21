# typikon

The Orthodox liturgical day: feasts, seasons and the fast, computed rather
than fetched. Twenty Orthodox jurisdictions and four Byzantine Catholic
churches, across the Julian, Revised Julian and Gregorian reckonings.

No network, no data files, no service to be up.

```sh
npm install typikon
```

```js
import { today } from "typikon";

const day = today({ jurisdiction: "rocor" });

day.churchDate;  // "5 September"
day.fast;        // "xerophagy"
day.fastLabel;   // "xerophagy — no oil or wine"
day.emoji;       // the glyph for the level
day.title;       // "The Holy Prophet Zachariah, father of the Forerunner"
day.season;      // null in ordinary time
day.daysToPascha;
```

In a browser or a bundler, load the module first:

```js
import init, { today } from "typikon";

await init();
const day = today({ jurisdiction: "rocor" });
```

Node needs no `init`, because it can read the WebAssembly itself. The exports
map picks the right build; `typikon/node` and `typikon/web` force one.

## What comes back

A plain object, not a handle: the same document `typikon --json` emits, with
the same field names and the same stability promise. Nothing has to be freed
and `JSON.stringify` works on it. TypeScript types ship with the package.

## The rest of it

```js
import { describe, describeIso, week, pascha, jurisdictions } from "typikon";

describe(new Date(2026, 3, 12), { jurisdiction: "moscow" });
describeIso("2026-04-12", { jurisdiction: "moscow" });

week(new Date(), 7, { jurisdiction: "goarch" });   // seven consecutive days
pascha(2027, "old");                               // 2027-05-02
jurisdictions();                                   // [{ key, name, note }, ...]
```

`week` reckons the year's movable cycle once rather than once a day, so it is
the thing to call for a calendar grid.

## Which calendar

`jurisdiction` takes a church key, or a bare calendar where the church does not
matter: `"old"` (Julian throughout), `"new"` (Revised Julian fixed feasts with
the Julian Paschalion, the default) or `"western"` (Gregorian throughout,
Pascha included). `jurisdictions()` lists the rest. Two more options:

- `tradition`, `"slavic"` or `"greek"`, overriding the church's own usage. It
  moves fish in the Nativity Fast and the rank of the Protection.
- `typikon: true`, to show a day at the Typikon's full discipline rather than
  at the minimum a church's own particular law requires. What binds is reported
  either way, in `binds` and `obligation`. This is for the Byzantine Catholic
  churches, whose law asks very much less than the tradition they keep.

Unlike the command line and the Python bindings, there is no remembered
default: a browser has nowhere to keep one, so the default is the New Calendar
and anything else is passed.

## Accuracy

The Paschalion is exact. The fast follows the Typikon as the standard printed
calendars apply it, which is the part where careful people differ; a parish
calendar or a confessor overrides this, and neither is a bug report.

## Licence

GPL-2.0-only. Source: https://git.pumpin.online/funkpower/typikon
