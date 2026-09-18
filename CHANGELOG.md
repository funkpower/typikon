# Changelog

Notable changes, in the terms [docs/versioning.md](docs/versioning.md) sets out.
**Calendar** entries change what a day reports, whatever the version number
does.

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
