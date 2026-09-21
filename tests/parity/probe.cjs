// Node, by require. The other half of Node's world, and the half an exports
// map gets wrong.
const { readFileSync, writeFileSync } = require("node:fs");
const { describeIso } = require("typikon");

const [probesPath, outPath] = process.argv.slice(2);
const out = {};
for (const [date, jurisdiction, typikon] of JSON.parse(readFileSync(probesPath, "utf8"))) {
  const d = describeIso(date, { jurisdiction, typikon });
  out[`${date}|${jurisdiction}|${typikon ? 1 : 0}`] = {
    date: d.date, weekday: d.weekday, jurisdiction: d.jurisdiction,
    calendar: d.calendar, churchDate: d.churchDate, season: d.season,
    cycle: d.cycle, week: d.week, feast: d.feast, rank: d.rank, sign: d.sign,
    tradition: d.tradition, title: d.title, commemorations: d.commemorations,
    fast: d.fast, fastLabel: d.fastLabel, emoji: d.emoji, reason: d.reason,
    obligation: d.obligation, binds: d.binds, pascha: d.pascha,
    daysToPascha: d.daysToPascha, julianOffset: d.julianOffset,
  };
}
writeFileSync(outPath, JSON.stringify(out));
