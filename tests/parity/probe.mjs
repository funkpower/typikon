// Node, by import.
import { readFileSync, writeFileSync } from "node:fs";
import { describeIso } from "typikon";
import { shape } from "./shape.mjs";

const [probesPath, outPath] = process.argv.slice(2);
const out = {};
for (const [date, jurisdiction, typikon] of JSON.parse(readFileSync(probesPath, "utf8"))) {
  out[`${date}|${jurisdiction}|${typikon ? 1 : 0}`] =
    shape(describeIso(date, { jurisdiction, typikon }));
}
writeFileSync(outPath, JSON.stringify(out));
