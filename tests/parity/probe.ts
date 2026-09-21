// TypeScript, compiled under --strict and then run. Every field is annotated
// on the way out, so a type that has drifted from what the package returns is
// a compile error rather than a surprise at run time.
import { readFileSync, writeFileSync } from "node:fs";
import { describeIso, week, pascha, jurisdictions, type Day, type Options } from "typikon";

const [probesPath, outPath] = process.argv.slice(2);
type Probe = [string, string, boolean];

const out: Record<string, Record<string, unknown>> = {};
for (const [date, jurisdiction, typikon] of JSON.parse(readFileSync(probesPath, "utf8")) as Probe[]) {
  const options: Options = { jurisdiction, typikon };
  const d: Day = describeIso(date, options);
  const season: string | null = d.season;
  const commemorations: string[] = d.commemorations;
  const daysToPascha: number = d.daysToPascha;
  out[`${date}|${jurisdiction}|${typikon ? 1 : 0}`] = {
    date: d.date, weekday: d.weekday, jurisdiction: d.jurisdiction,
    calendar: d.calendar, churchDate: d.churchDate, season,
    cycle: d.cycle, week: d.week, feast: d.feast, rank: d.rank, sign: d.sign,
    tradition: d.tradition, title: d.title, commemorations,
    fast: d.fast, fastLabel: d.fastLabel, emoji: d.emoji, reason: d.reason,
    obligation: d.obligation, binds: d.binds, pascha: d.pascha,
    daysToPascha, julianOffset: d.julianOffset,
  };
}

// The rest of the surface, so nothing is typed only where it is called.
const seven: Day[] = week(new Date(2026, 3, 12), 7, { jurisdiction: "moscow" });
const p: Date = pascha(2027, "old");
const keys: string[] = jurisdictions().map((j) => j.key);
if (seven.length !== 7 || p.getFullYear() !== 2027 || keys.length < 20) {
  throw new Error("the rest of the surface did not answer");
}

writeFileSync(outPath, JSON.stringify(out));
