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

- A saint of vigil or polyeleos rank now relaxes a weekday of a fast: to wine
  and oil inside the Great Fast, as far as fish in the Nativity, Apostles' and
  Dormition Fasts. 74 days across 2020 to 2030 had been reported stricter than
  any published calendar keeps them, St Nicholas and the Forty Martyrs of
  Sebaste among them.
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
