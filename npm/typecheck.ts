// The published types, checked the way a TypeScript consumer would meet them.
// Run by npm/test.mjs. Nothing here is executed; the point is that `tsc
// --strict` accepts it, and rejects the lines below that it should.
import {
  describe,
  describeIso,
  today,
  week,
  pascha,
  jurisdictions,
  version,
  type Day,
  type Options,
  type JurisdictionInfo,
} from "typikon";

const opts: Options = { jurisdiction: "rocor", tradition: "slavic", typikon: true };

const day: Day = describeIso("2026-04-12", opts);
const byDate: Day = describe(new Date(), { jurisdiction: "moscow" });
const now: Day = today();

const churchDate: string = day.churchDate;
const season: string | null = day.season;
const commemorations: string[] = day.commemorations;
const daysToPascha: number = day.daysToPascha;

const seven: Day[] = week(new Date(), 7, opts);
const p: Date = pascha(2027, "old");
const all: JurisdictionInfo[] = jurisdictions();
const v: string = version();

// Referenced so nothing is unused, which --strict would otherwise let pass but
// a linter would not.
export const _ = { byDate, now, churchDate, season, commemorations, daysToPascha, seven, p, all, v };

// @ts-expect-error a season may be absent, so it is not a plain string
const wrong: string = day.season;
// @ts-expect-error there is no such option
const badOption: Options = { jurisdictionn: "rocor" };
// @ts-expect-error days is a number
const badWeek = week(new Date(), "7", opts);

export const __ = { wrong, badOption, badWeek };
