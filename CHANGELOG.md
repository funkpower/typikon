# Changelog

Notable changes, in the terms [docs/versioning.md](docs/versioning.md) sets out.
**Calendar** entries change what a day reports, whatever the version number
does.

## Unreleased

### Fixed

- The npm package's files are reachable through its exports map. A bundler
  asked for `typikon/web/typikon_bg.wasm` was told no such path is exported,
  so a web app that wants to hand the WebAssembly to `init()` as an asset had
  to reach around the package into `node_modules`. The documented call, `await
  init()` with no argument, was never affected.

### Added

- `tests/parity.sh`: every binding asked the same twelve days and compared
  field by field against the command line. The days are the ones where the
  bindings could plausibly differ, and the surfaces are the command line, the
  Python wheel, Node by import and by require, TypeScript under `--strict`, a
  real browser, Deno, Bun, the C ABI and LuaJIT over it. A surface whose
  toolchain is absent is named as skipped rather than silently counted.
  `--published` runs the same checks against PyPI and npm instead of the
  checkout.

### Changed

- A release runs what a push runs. The release workflow had been gated on
  `cargo test` alone, so the C ABI, the Lua binding, the Python parity check
  and the README examples were checked on the branch that led to a release but
  not on the release itself. A version cannot be reused on either registry, so
  the gate belongs before the upload.

## 1.1.0

New ways in, and no change to any day: nothing here moves a feast, a fast
or a commemoration.

### Added

- A C ABI, behind the `capi` feature, with a header in `include/typikon.h`.
  Dates cross it as Julian Day Numbers, so no struct layout is part of the
  contract. A day is opaque and owns its strings: an accessor returns the same
  borrowed pointer every time and allocates nothing, so reading one in a render
  loop is free. The accessors carry every field of the JSON, so a statusline
  does not need a parser. Errors go to a thread-local `typikon_last_error()`.
- A LuaJIT FFI binding in `examples/lua/typikon.lua`, for AwesomeWM, WezTerm
  and Neovim, and a worked C example in `examples/c/demo.c`.
- `jurisdictions::resolve_full`, which resolves a jurisdiction together with
  the usage and the discipline. The Python bindings and the C ABI now share it
  rather than each keeping a copy, and it interns the combinations instead of
  leaking one per call.
- A JavaScript and TypeScript package, behind the `wasm` feature and published
  to npm as `typikon`. One WebAssembly build serves Node, Deno, Bun and the
  browser. A day comes back as a plain object rather than a handle: the same
  document `--json` emits, with nothing to free. The types are hand-written
  rather than inferred, and `tsc --strict` is a release gate the way `mypy
  --strict` already is. There is no remembered default jurisdiction there,
  because a browser has nowhere to keep one; the default is the New Calendar.

## 1.0.0

First stable release. The public surface is now the one described in
[docs/versioning.md](docs/versioning.md), and breaking it requires a major
version.

### Calendar

Everything below changes what a day reports against pre-1.0 versions.

- A ranked saint now relaxes a weekday of a fast. A saint of vigil or polyeleos
  rank reaches wine and oil; a great feast reaches fish in the Nativity and
  Apostles' Fasts and wine and oil elsewhere. No saint of any rank brings fish
  into the Great Fast or the Dormition Fast. Days that had been reported
  stricter than any published calendar keeps them now are not, St Nicholas on a
  Wednesday of the Nativity Fast and the Forty Martyrs of Sebaste among them;
  on the new calendar it changes 104 days between 1 January 2020
  and 31 December 2030.
- The Annunciation is tested before Great Friday and Great Saturday. In the
  years it falls on either it had been absorbed into them: reported as a total
  fast, demoted to a bullet, and its rank overwritten. It now lifts those days
  to wine and oil.
- Lazarus Saturday and Palm Sunday belong to the Great Fast season. They had
  belonged to none, so rules that key on the season fell through and the
  Byzantine Catholic jurisdictions reported no fast at all on Palm Sunday.
- The eves of the Nativity and Theophany relax to wine and oil when they fall
  on a Saturday or Sunday.
- Greek usage admits fish on any day of the Nativity Fast but Wednesday and
  Friday until 17 December. Slavic usage keeps fish to Saturdays and Sundays.
- The Protection of the Theotokos is a great feast in Slavic usage.
- The Nativity of the Forerunner is ranked a great feast.
- Melkite, Romanian Greek Catholic and Ukrainian Greek Catholic jurisdictions
  no longer report any day as binding. They had borrowed Ruthenian particular
  law, which is not their law.

### Hardened

- The years this calendar answers for are 1 to 9999, enforced. `Date::from_ymd`
  and `from_jdn` reject anything outside them, arithmetic saturates rather than
  wrapping, and a seven-day walk stops at the boundary instead of running past
  it into dates whose ISO form was not ten characters long.
- `--days` is bounded at a hundred years and `--indent` at sixteen. Both took
  any integer and aborted the process on allocation.
- `--update-menaion` bounds the day it parses by the month it is in, and
  refuses to write when most of the year comes back empty. The page is fetched,
  so its contents are not this program's to trust: an unbounded parse wrote
  keys no lookup could match, and a page redesign would have blanked a year of
  commemorations in a calendar that had been working.
- `today()` says on stderr when it cannot read the local offset, rather than
  silently reporting UTC and so the wrong day either side of midnight.
- The Ukrainian Greek Catholic Church no longer loads the Ruthenian calendar of
  saints. It is Pittsburgh's, and lending it was the same error as lending the
  law.

### Interfaces

- `binds` is null unless the day binds, rather than the string `"none"`. Most
  jurisdictions legislate no binding fast at all, which is a different fact
  from a day that happens to oblige nothing.
- iCalendar UIDs carry the tradition and the discipline. Two feeds of the same
  day that say different things were sharing an identity, so a client treated
  them as one event.
- `Date`'s Julian Day Number is behind `jdn()` and `from_jdn()` rather than a
  public field.
- `fastLabel`, `emoji` and `reason` are documented as presentation: the fields
  are stable, their wording is not.

### Added

- Greek and Slavic usage as an explicit axis, per jurisdiction and overridable
  with `--tradition`.
- `--typikon` and `--no-typikon`: show a day at the Typikon's full discipline
  while still reporting what a church's own particular law obliges.
- `--format csv` and `--format tsv`, with `--fields` and `--list-fields`.
- `--no-emoji`.
- The rank marks the liturgical books print, as the `sign` field.
- ACROD, the American Carpatho-Russian Orthodox Diocese.
- `--set-default` remembers the tradition and the discipline alongside the
  jurisdiction.

### Fixed

- The seven-day view no longer panics. It cut a title at a byte index into a
  string it had counted in characters, and every movable feast title carries an
  em dash, so at fourteen terminal widths, 80 among them, it aborted with exit
  101. Every title the program can print is now cut at every width it can be
  asked to print at, in a test.
- The Python bindings honour `TYPIKON_JURISDICTION`. They read the config file
  where the command line reads the environment first, so the two answered for
  different churches on the same machine.

### Packaging

- Requires Rust 1.85, which is what the dependency tree has needed since clap
  4.6. `rust-version` had said 1.74, so the source distribution claimed to
  build on toolchains where cargo refuses outright.
- Classified as production/stable, with the Python version and platform
  classifiers PyPI indexes on, and Changelog, Documentation and Issues links.
- README links are absolute, because PyPI does not resolve relative ones and
  every one of them 404'd on the project page.

### Changed

- Rewritten in Rust. It ships as one static binary with the menologion compiled
  in, and as an abi3 wheel. The port was verified by running it against the
  previous implementation over 199,446 day-records, 26 jurisdictions across 21
  years, requiring every field of every record to match.
- iCalendar folds content lines on octets, which is what RFC 5545 counts. It
  had folded on characters, so a line with emoji ran up to 80 octets against a
  75-octet limit.
- The seven-day view elides a long commemoration on a word boundary and takes
  its width from the terminal, rather than cutting at 44 characters mid-word.
- A malformed `--date` or an out-of-range `--year` is a usage error, not a
  traceback.
