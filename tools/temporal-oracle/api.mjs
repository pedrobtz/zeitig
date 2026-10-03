// Prints the share of the Temporal API that zeitig implements: every static
// and prototype member of the polyfill's Temporal classes must have a row in
// api-map.csv naming the zeitig equivalent, "-" when there is none, or "n/a"
// for JavaScript mechanics with no R counterpart (valueOf), which are not
// counted. Exits with an error when a member is missing from the map.
//
// Usage (from this directory): node api.mjs

import { Temporal } from "@js-temporal/polyfill";
import { readFileSync } from "node:fs";

const map = new Map(
  readFileSync(new URL("./api-map.csv", import.meta.url), "utf8")
    .trim().split("\n").slice(1)
    .map((line) => {
      const i = line.indexOf(",");
      return [line.slice(0, i), line.slice(i + 1).replace(/^"|"$/g, "")];
    }),
);

const members = [];
for (const name of Object.getOwnPropertyNames(Temporal).sort()) {
  const C = Temporal[name];
  if (typeof C !== "function" && typeof C !== "object") continue;
  for (const m of Object.getOwnPropertyNames(C).sort()) {
    if (!["length", "name", "prototype"].includes(m)) members.push(`${name}.${m}`);
  }
  for (const m of Object.getOwnPropertyNames(C.prototype ?? {}).sort()) {
    if (m !== "constructor") members.push(`${name}.prototype.${m}`);
  }
}

const missing = members.filter((m) => !map.has(m));
if (missing.length) {
  console.error(`members missing from api-map.csv:\n  ${missing.join("\n  ")}`);
  process.exit(1);
}
const counted = members.filter((m) => map.get(m) !== "n/a");
const done = counted.filter((m) => map.get(m) !== "-");
console.log(`implemented: ${done.length} of ${counted.length} members ` +
  `(${(100 * done.length / counted.length).toFixed(0)}%)`);
const byClass = {};
for (const m of counted) {
  const c = m.split(".")[0];
  byClass[c] ??= [0, 0];
  byClass[c][1]++;
  if (map.get(m) !== "-") byClass[c][0]++;
}
for (const [c, [d, n]] of Object.entries(byClass)) console.log(`  ${c}: ${d} of ${n}`);
