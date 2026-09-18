// The npm package, exercised as a consumer would use it, against the same
// answers the command line gives.
//
//     npm/build.sh && node npm/test.mjs
//
// It imports by package path through a temporary node_modules link, so what is
// tested is the exports map and the built files, not a lucky relative import.

import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtempSync, symlinkSync, mkdirSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, dirname, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, "..");
const cli = join(root, "target", "release", "typikon");

const stage = mkdtempSync(join(tmpdir(), "typikon-npm-"));
mkdirSync(join(stage, "node_modules"));
symlinkSync(join(root, "npm", "dist"), join(stage, "node_modules", "typikon"), "dir");

// Resolve "typikon" from inside the staged directory, so the exports map is
// what decides which build loads.
const { createRequire } = await import("node:module");
const req = createRequire(join(stage, "probe.js"));
const entry = req.resolve("typikon");
const typikon = await import(pathToFileURL(entry).href);

let failures = 0;
function check(name, fn) {
  try {
    fn();
    console.log(`  ok   ${name}`);
  } catch (e) {
    failures++;
    console.log(`  FAIL ${name}\n       ${e.message}`);
  }
}

console.log(`typikon ${typikon.version()} from ${entry}`);

check("node resolves to the node build", () => {
  assert.match(entry, /dist\/node\/typikon\.js$/);
});

check("a day is a plain object, not a handle", () => {
  const day = typikon.describeIso("2026-04-12", { jurisdiction: "moscow" });
  assert.equal(Object.getPrototypeOf(day), Object.prototype);
  assert.equal(typeof day.free, "undefined");
  assert.doesNotThrow(() => JSON.stringify(day));
});

check("it answers what the command line answers", () => {
  for (const [iso, jurisdiction] of [
    ["2026-04-12", "moscow"],
    ["2026-01-07", "goarch"],
    ["2026-02-16", "ruthenian"],
    ["2025-12-25", "old"],
    ["2026-08-15", "antiochian"],
  ]) {
    const expected = JSON.parse(
      execFileSync(cli, ["--date", iso, "--jurisdiction", jurisdiction, "--json"], {
        encoding: "utf8",
        env: { ...process.env, XDG_CONFIG_HOME: stage, XDG_DATA_HOME: stage },
      }),
    );
    const got = typikon.describeIso(iso, { jurisdiction });
    assert.deepEqual(got, expected, `${iso} ${jurisdiction}`);
  }
});

check("a year of days matches the command line, day for day", () => {
  // Five dates prove the wiring; a year proves the engine crossed the boundary
  // whole. The CLI emits an array for --days, so this is one process per
  // jurisdiction rather than one per day.
  for (const [jurisdiction, opts] of [
    ["moscow", {}],
    ["goarch", {}],
    ["ruthenian", { typikon: true }],
    ["finnish", {}],
  ]) {
    const args = ["--date", "2026-01-01", "--days", "365", "--jurisdiction", jurisdiction, "--json"];
    if (opts.typikon) args.push("--typikon");
    const expected = JSON.parse(
      execFileSync(cli, args, {
        encoding: "utf8",
        maxBuffer: 64 << 20,
        env: { ...process.env, XDG_CONFIG_HOME: stage, XDG_DATA_HOME: stage },
      }),
    ).days; // a run of days is wrapped, a single day is not
    const got = typikon.week(new Date(2026, 0, 1), 365, { jurisdiction, ...opts });
    assert.equal(got.length, expected.length, jurisdiction);
    for (let i = 0; i < expected.length; i++) {
      assert.deepEqual(got[i], expected[i], `${jurisdiction} ${expected[i].date}`);
    }
  }
});

check("a Date and an ISO string agree", () => {
  // JS months are zero-based, which is the single easiest thing to get wrong
  // on this boundary.
  const a = typikon.describe(new Date(2026, 3, 12), { jurisdiction: "moscow" });
  const b = typikon.describeIso("2026-04-12", { jurisdiction: "moscow" });
  assert.deepEqual(a, b);
});

check("options are optional", () => {
  const day = typikon.today();
  assert.equal(day.jurisdiction, "new");
});

check("a week is consecutive and matches day by day", () => {
  const days = typikon.week(new Date(2026, 3, 12), 7, { jurisdiction: "moscow" });
  assert.equal(days.length, 7);
  assert.equal(days[0].date, "2026-04-12");
  assert.equal(days[6].date, "2026-04-18");
  for (const d of days) {
    assert.deepEqual(d, typikon.describeIso(d.date, { jurisdiction: "moscow" }));
  }
});

check("pascha matches the published dates", () => {
  const iso = (d) =>
    `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
  assert.equal(iso(typikon.pascha(2026, "old")), "2026-04-12");
  assert.equal(iso(typikon.pascha(2026, "western")), "2026-04-05");
  assert.equal(iso(typikon.pascha(2027, "old")), "2027-05-02");
});

check("every listed jurisdiction reckons", () => {
  const all = typikon.jurisdictions();
  assert.ok(all.length > 20);
  for (const j of all) {
    assert.equal(typeof j.key, "string");
    assert.equal(typeof j.name, "string");
    const day = typikon.describeIso("2026-04-12", { jurisdiction: j.key });
    assert.equal(day.jurisdiction, j.key);
  }
});

check("the discipline flag moves what is shown but not what binds", () => {
  const law = typikon.describeIso("2026-03-04", { jurisdiction: "ruthenian", typikon: false });
  const full = typikon.describeIso("2026-03-04", { jurisdiction: "ruthenian", typikon: true });
  assert.notEqual(law.fast, full.fast);
  assert.equal(law.binds, full.binds);
});

check("the usage can be overridden", () => {
  const greek = typikon.describeIso("2025-12-09", { jurisdiction: "old", tradition: "greek" });
  const slavic = typikon.describeIso("2025-12-09", { jurisdiction: "old", tradition: "slavic" });
  assert.equal(greek.tradition, "greek");
  assert.equal(slavic.tradition, "slavic");
  assert.notEqual(greek.fast, slavic.fast);
});

check("a jurisdiction nobody keeps throws", () => {
  assert.throws(() => typikon.describeIso("2026-04-12", { jurisdiction: "shire" }), /shire/);
});

check("a date outside the years throws", () => {
  assert.throws(() => typikon.describeIso("not-a-date"));
  assert.throws(() => typikon.describe(new Date(-5000, 0, 1)));
});

check("the web build is there", () => {
  assert.match(req.resolve("typikon/web"), /dist\/web\/typikon\.js$/);
});

// The web build, driven the way a browser drives it. Node can hand it the
// bytes, which is enough to prove the glue and the module agree; what it
// cannot prove is that a browser will fetch them, and no test here can.
{
  const webEntry = req.resolve("typikon/web");
  const web = await import(pathToFileURL(webEntry).href);
  const { readFileSync } = await import("node:fs");
  await web.default({
    module_or_path: readFileSync(join(dirname(webEntry), "typikon_bg.wasm")),
  });
  check("the web build answers what the node build answers", () => {
    assert.deepEqual(
      web.describeIso("2026-04-12", { jurisdiction: "moscow" }),
      typikon.describeIso("2026-04-12", { jurisdiction: "moscow" }),
    );
    assert.equal(web.version(), typikon.version());
  });
}

// TypeScript, if it is to hand. The Python bindings are gated on `mypy
// --strict`; this is the same gate, and the same reason: a published type is a
// promise, and a promise nobody checks drifts.
{
  const tsc = process.env.TSC ?? "tsc";
  let have = true;
  try {
    execFileSync(tsc, ["--version"], { stdio: "ignore" });
  } catch {
    have = false;
    console.log("  skip typescript --strict (no tsc on PATH; set TSC=)");
  }
  if (have) {
    const { copyFileSync } = await import("node:fs");
    copyFileSync(join(here, "typecheck.ts"), join(stage, "typecheck.ts"));
    check("the published types pass tsc --strict", () => {
      execFileSync(
        tsc,
        ["--strict", "--noEmit", "--target", "es2022", "--module", "nodenext",
         "--moduleResolution", "nodenext", "typecheck.ts"],
        { cwd: stage, stdio: "pipe", encoding: "utf8" },
      );
    });
  }
}

rmSync(stage, { recursive: true, force: true });
console.log(failures ? `\n${failures} failed` : "\nall passed");
process.exit(failures ? 1 : 0);
