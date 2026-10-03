// Generates tests/testthat/fixtures/temporal/*.json: the expected Temporal
// results for zeitig's conformance test (tests/testthat/test-temporal-conformance.R),
// computed with the reference polyfill pinned in package.json.
//
// Usage (from this directory): npm ci && node generate.mjs
//
// The output is deterministic: rerunning without changes gives no diff.

import { Temporal } from "@js-temporal/polyfill";
import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { divergence } from "./divergences.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const outDir = join(here, "..", "..", "tests", "testthat", "fixtures", "temporal");
const polyfill = JSON.parse(
  readFileSync(join(here, "node_modules", "@js-temporal", "polyfill", "package.json"), "utf8"),
).version;

const CLASSES = {
  "plain-date": Temporal.PlainDate,
  "plain-time": Temporal.PlainTime,
  "plain-date-time": Temporal.PlainDateTime,
  "instant": Temporal.Instant,
  "zoned-date-time": Temporal.ZonedDateTime,
  "duration": Temporal.Duration,
};

const str = (x) => (x === null || x === undefined ? "null" : String(x));

// Each op takes the Temporal class and the case and returns the result as a
// string (Temporal's toString() format, or String() for numbers and booleans).
const OPS = {
  from: (C, c) => C.from(c.receiver, c.options).toString(),
  from_fields: (C, c) => C.from(c.args[0], c.options).toString(),
  add: (C, c) => C.from(c.receiver).add(c.args[0], c.options).toString(),
  subtract: (C, c) => C.from(c.receiver).subtract(c.args[0], c.options).toString(),
  until: (C, c) => C.from(c.receiver).until(c.args[0], c.options).toString(),
  since: (C, c) => C.from(c.receiver).since(c.args[0], c.options).toString(),
  round: (C, c) => C.from(c.receiver).round(c.options).toString(),
  with: (C, c) => C.from(c.receiver).with(c.args[0], c.options).toString(),
  toString: (C, c) => C.from(c.receiver).toString(c.options),
  compare: (C, c) => str(C.compare(c.receiver, c.args[0], c.options)),
  equals: (C, c) => str(C.from(c.receiver).equals(c.args[0])),
  get: (C, c) => str(C.from(c.receiver)[c.args[0]]),
  negated: (C, c) => C.from(c.receiver).negated().toString(),
  abs: (C, c) => C.from(c.receiver).abs().toString(),
  total: (C, c) => str(C.from(c.receiver).total(c.options)),
  withTimeZone: (C, c) => C.from(c.receiver).withTimeZone(c.args[0]).toString(),
  startOfDay: (C, c) => C.from(c.receiver).startOfDay().toString(),
  getTimeZoneTransition: (C, c) => str(C.from(c.receiver).getTimeZoneTransition(c.args[0])),
  toZonedDateTime: (C, c) => C.from(c.receiver).toZonedDateTime(c.args[0], c.options).toString(),
  toZonedDateTimeISO: (C, c) => C.from(c.receiver).toZonedDateTimeISO(c.args[0]).toString(),
  toInstant: (C, c) => C.from(c.receiver).toInstant().toString(),
  toPlainDateTime: (C, c) => C.from(c.receiver).toPlainDateTime().toString(),
  fromEpochNanoseconds: (C, c) => C.fromEpochNanoseconds(BigInt(c.args[0])).toString(),
  fromEpochMilliseconds: (C, c) => C.fromEpochMilliseconds(c.args[0]).toString(),
};

function run(type, c) {
  const fn = OPS[c.op];
  if (!fn) throw new Error(`unknown op ${c.op}`);
  try {
    return { result: fn(CLASSES[type], c), error: null };
  } catch (e) {
    if (!(e instanceof RangeError || e instanceof TypeError)) throw e;
    return { result: null, error: e.constructor.name };
  }
}

// Serialise with one case per line and keys in a fixed order.
function serialise(type, cases) {
  const lines = cases.map((c) => "    " + JSON.stringify(c));
  return [
    "{",
    `  "polyfill": "@js-temporal/polyfill ${polyfill}",`,
    `  "type": "${type}",`,
    `  "cases": [`,
    lines.join(",\n"),
    "  ]",
    "}",
    "",
  ].join("\n");
}

const TYPES = ["plain-date", "plain-time", "plain-date-time", "duration", "instant",
  "zoned-date-time"];

mkdirSync(outDir, { recursive: true });
const extra = [
  ...(await import("./cases/regressions.mjs")).default(),
  ...(await import("./cases/test262.mjs")).default(),
];
let total = 0;
for (const type of TYPES) {
  const gen = (await import(`./cases/${type}.mjs`)).default;
  const raw = [...gen(), ...extra.filter((c) => c.type === type)];
  const seen = new Set();
  const counters = new Map();
  const cases = [];
  for (const c of raw) {
    const key = JSON.stringify([c.op, c.receiver, c.args, c.options]);
    if (seen.has(key)) continue;
    seen.add(key);
    const { result, error } = run(type, c);
    counters.set(c.op, (counters.get(c.op) ?? 0) + 1);
    const rec = { id: `${c.op}/${String(counters.get(c.op)).padStart(4, "0")}`, op: c.op };
    if (c.receiver !== undefined) rec.receiver = c.receiver;
    if (c.args !== undefined) rec.args = c.args;
    if (c.options !== undefined) rec.options = c.options;
    rec.result = result;
    rec.error = error;
    const div = divergence(type, rec, (x) => run(type, { ...c, ...x }));
    if (div) Object.assign(rec, div);
    // Exactly one of `result` and `error` is kept.
    if (error === null) delete rec.error;
    else delete rec.result;
    if (c.source) rec.source = c.source;
    cases.push(rec);
  }
  writeFileSync(join(outDir, `${type}.json`), serialise(type, cases));
  total += cases.length;
  console.log(`${type}: ${cases.length} cases`);
}
console.log(`total: ${total} cases`);
