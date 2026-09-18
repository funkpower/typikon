// The fields, named once for every JavaScript surface. Written out rather than
// spread, so that a field the package stops exporting is a missing key here and
// not a silently absent one.
export function shape(d) {
  return {
    date: d.date, weekday: d.weekday, jurisdiction: d.jurisdiction,
    calendar: d.calendar, churchDate: d.churchDate, season: d.season,
    cycle: d.cycle, feast: d.feast, rank: d.rank, sign: d.sign,
    tradition: d.tradition, title: d.title, commemorations: d.commemorations,
    fast: d.fast, fastLabel: d.fastLabel, emoji: d.emoji, reason: d.reason,
    obligation: d.obligation, binds: d.binds, pascha: d.pascha,
    daysToPascha: d.daysToPascha, julianOffset: d.julianOffset,
  };
}
