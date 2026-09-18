# Versioning

`typikon` follows semantic versioning from 1.0.0 onward. This page says what
that covers, because a calendar has a wider public surface than a library
usually does: four things depend on it, and only one of them is a function
signature.

## What is stable

A **major** version is required to change any of these.

### The JSON

Field names, their types, and whether a field can be null. Front ends are the
thing most likely to be written by someone else, and a schema that shifts
underneath them is a bad neighbour. As of 1.0.0:

```
date weekday jurisdiction calendar churchDate season cycle feast rank sign
tradition title commemorations fast fastLabel emoji reason obligation binds
pascha daysToPascha julianOffset
```

Adding a field is a **minor** change, so read the object by key and ignore what
you do not recognise. `season`, `cycle`, `feast`, `rank`, `sign` and
`obligation` are null on ordinary days.

Two semantics worth stating because the names do not carry them:

- **`daysToPascha` is signed**, and counts to the Pascha of that day's own
  civil year. It is negative after Pascha has passed. 31 December reports
  about -263, not the days until the next one.
- **`julianOffset`** is reported for every jurisdiction, including those that
  do not use the Julian menologion.

### The fasting levels

`strict` `xerophagy` `wine_oil` `fish` `dairy` `none`, and their order from
strictest to laxest. Code that compares them may rely on that order.

### The jurisdiction and calendar keys

`old` `new` `western`, and every jurisdiction key `typikon --list` prints.
A key is never renamed or removed in a minor version. New ones are added
freely.

### The command line

Flag names and what they do, the exit codes (0 success, 2 usage error), and the
column names `--format csv` accepts. `--json` and `--ics` remain as shorthand
for `--format`.

### The Rust and Python APIs

Public items in the crate root and the modules `calendars`, `paschalion`,
`seasons`, `fasting`, `jurisdictions`, `menologion`, `day`, `export`, `config`;
and in Python, the module's functions and the attributes of `Day`.

`Day.to_dict()` in Python carries the same fields as the JSON, in snake_case
rather than camelCase, which is the one deliberate difference between them.

## What is not stable

- **The text output.** It is for a person to read, and it changes when a better
  arrangement is found. Parse the JSON, CSV or TSV instead.
- **The `reason` strings.** They explain a decision to a reader; they are not
  identifiers. Key on `fast`, `season` and `obligation`.
- **The menologion.** Commemorations get added, corrected and reworded. See
  below.
- **Anything not listed above**, including every private item and the layout of
  fetched menaia on disk.

## Corrections to the calendar itself

This is the category semver does not have a word for, and it is the one that
matters most here.

A correction to a fast rule, a feast rank or a commemoration changes what the
program tells you about a day. It breaks no interface, so by semver it is a
patch. But it changes what someone is told to eat, and a user who has noticed a
disagreement with their parish deserves to know it moved.

So: **every change to what a day reports is called out in the changelog under
"Calendar"**, with the days affected, whatever the version number does. If a
correction changes a day you care about, the changelog is where it will say so.

Adding a jurisdiction, a menaion or a tradition is a minor version. Correcting
an existing one is a patch, and is announced.

## Releasing

See [releasing.md](releasing.md). The version lives only in `Cargo.toml`.
