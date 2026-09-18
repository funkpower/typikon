# The state of the calendars

Which churches keep which reckoning, and what this program models of each.

## Three independent questions

Almost every disagreement about the date of a feast comes from conflating
these. They vary separately.

**The paschalion** decides Pascha and everything counted from it: the Triodion,
Great Week, the Pentecostarion, the Apostles' Fast. Two tables are in use. The
**Julian Paschalion** is the old computus, worked in the Julian calendar and
read onto a civil date. The **Gregorian Paschalion** is the Western one. They
agree in some years and can diverge by five weeks.

**The menologion** decides the fixed feasts: the Nativity, Theophany, the
Dormition, and every saint's day. Two calendars are in use. The **Julian**
menologion runs thirteen days behind the civil calendar until 2100 and fourteen
after it. The **Revised Julian** menologion agrees with the Gregorian for every
date until 2800, so in practice it is the civil calendar.

**The usage** decides rank and some of the fast: Greek books and Slavic books
read the same Typikon a little differently.

The three combinations in use have short names:

| | paschalion | menologion | kept by |
|---|---|---|---|
| **old** | Julian | Julian | Moscow, ROCOR, Serbia, Georgia, Jerusalem, the Holy Mountain, Poland, Macedonia, ACROD |
| **new** | Julian | Revised Julian | Constantinople, Greece, Antioch, the OCA, Romania, Bulgaria, Cyprus, Albania, Ukraine |
| **western** | Gregorian | Revised Julian | the Finnish Orthodox Church; most Byzantine Catholic churches |

A new-calendar church therefore keeps the Nativity with the West and Pascha
with the East. That is not a compromise but what the 1923 congress at
Constantinople decided: adopt the Revised Julian menologion, leave the
paschalion alone.

## How it came about

The Pan-Orthodox Congress of May 1923 proposed the Revised Julian calendar.
Constantinople, Greece, Romania, Poland and others adopted it over the
following years; Alexandria, Antioch, Bulgaria and Cyprus followed later.
Jerusalem, Russia, Serbia, Georgia and the Holy Mountain did not, and have
not.

Three changes since are worth knowing because they are recent enough to catch
out a printed reference:

- **1923.** The Council of Bishops of the Russian Church Abroad rejected the
  new calendar outright, which is why ROCOR is Julian throughout and why that
  became part of its identity rather than a regional custom.
- **2014.** The Polish Orthodox Church returned to the Julian menologion after
  decades on the Revised Julian. Some parishes kept the new calendar.
- **1 September 2023.** The Ukrainian Greek Catholic Church and the Orthodox
  Church of Ukraine moved their fixed feasts to the Revised Julian calendar,
  keeping the Julian Paschalion.

One Orthodox church keeps the Gregorian Paschalion: the Finnish Orthodox
Church, which has done so since the 1920s.

## The Orthodox churches

All twenty are modelled, with the Typikon's own discipline. The Orthodox Church
legislates fasting as tradition rather than as precept, so nothing is reported
as binding for any of them.

| jurisdiction | reckoning | usage |
|---|---|---|
| `moscow` `rocor` `serbian` `georgian` `jerusalem` `athos` `polish` `macedonian` `acrod` | old | Slavic, except `georgian` `jerusalem` `athos` |
| `constantinople` `goarch` `greece` `antiochian` `cyprus` `albanian` `romanian` | new | Greek |
| `oca` `bulgarian` `ukrainian` | new | Slavic |
| `finnish` | western | Greek |

Where a church is not uniform, the jurisdiction's note says so: most OCA
parishes keep the new calendar and a minority of Carpatho-Russian and Alaskan
background keep the old; ACROD publishes both cycles and its parishes keep one
or the other; Polish parishes vary since 2014.

## The Byzantine Catholic churches

Fourteen Eastern Catholic churches use the Byzantine rite. Four have entries.

| church | reckoning | particular law on fasting |
|---|---|---|
| `ruthenian`, the Metropolia of Pittsburgh | western | modelled, from the Metropolia's own published rule |
| `ugcc`, Ukrainian Greek Catholic | new since 2023 | its own; not modelled |
| `melkite`, Melkite Greek Catholic | western | its own; not modelled |
| `romanian-catholic`, Romanian Greek Catholic | western | its own; not modelled |

The other ten are the Slovak, Hungarian, Križevci (Croatia and Serbia),
Italo-Albanian, Greek, Bulgarian, Macedonian, Belarusian, Russian and Albanian
Greek Catholic churches. They have no entry because their reckonings are not
uniform and are not recorded in general reference: several vary by eparchy, and
the churches that sit beside a local Orthodox majority often keep that
majority's paschalion for the sake of a common Pascha, as Melkite parishes in
the Middle East do.

Any of them keeps one of the three reckonings above, and two questions decide
which:

1. Is Pascha kept with Rome, or with the local Orthodox?
2. Are the fixed feasts on the civil calendar, or thirteen days behind it?

| | | |
|---|---|---|
| with Rome | civil calendar | `--calendar western` |
| with the local Orthodox | civil calendar | `--calendar new` |
| with the local Orthodox | thirteen days behind | `--calendar old` |

Add `--tradition slavic` or `--tradition greek` for the usage the books follow.
A patch adding a church, with a source for its calendar, is welcome.

### Why the law is never borrowed

Every Eastern Catholic church legislates its own fasting discipline, and they
differ. Ruthenian particular law binds on four kinds of day in the year:
strict abstinence on the first day of the Great Fast and on Great and Holy
Friday, and abstinence from meat on the Wednesdays and Fridays of the Great
Fast. That is the Metropolia of Pittsburgh's law and is reported for that
church alone.

The others are shown at the Typikon's discipline with nothing reported as
binding. Telling a Melkite that fourteen days a year oblige him under a law
written for Pittsburgh would be a false statement about his obligations, which
is a worse failure than saying nothing. When a church's law is sourced, its
entry will carry it.

`--typikon` shows any of these churches at the full Byzantine discipline while
still reporting whatever its own law obliges. For a Byzantine Catholic that is
usually the flag you want: the law is the floor, not the fast.

## Not in scope

Nine Eastern Catholic churches do not use the Byzantine rite: the Maronite,
Chaldean, Syro-Malabar, Syro-Malankara, Syriac, Armenian, Coptic, Ethiopian and
Eritrean churches. They have their own rites, their own sanctorals and their own
fasting structures, and none of the machinery here applies to them: there is no
Triodion, no Pentecostarion, no Byzantine menologion. The Latin Church likewise.

Supporting any of them would be a different program sharing nothing but the
word "calendar".
