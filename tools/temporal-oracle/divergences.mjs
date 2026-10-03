// Documented differences between zeitig and Temporal: each key is a row of
// the table in design.md section 9 (column "Key").
//
// divergence(type, record, rerun) returns undefined when zeitig must give
// Temporal's result, or {divergence, zeitig_result?, zeitig_error?}: the key
// and what zeitig gives instead. A divergent case without a zeitig_* field is
// not checked, only counted. `rerun(patch)` evaluates the case again with
// some fields replaced, to derive zeitig's result from Temporal's.

import { Temporal } from "@js-temporal/polyfill";

const CALENDAR_TYPES = new Set(["plain-date", "plain-date-time", "zoned-date-time"]);

// jiff's limits: civil dates in years -9999..9999, Timestamp::MIN/MAX, and
// the largest value of each Span unit.
const INSTANT_MIN = -377705023201n * 1000000000n;
const INSTANT_MAX = 253402207200n * 1000000000n + 999999999n;
const SPAN_MAX = {
  years: 19998, months: 239976, weeks: 1043497, days: 7304484, hours: 175307616,
  minutes: 10518456960, seconds: 631107417600, milliseconds: 631107417600000,
  microseconds: 631107417600000000, nanoseconds: 9223372036854775807,
};

function strings(rec) {
  const out = [];
  const add = (x) => {
    if (typeof x === "string") out.push(x);
    else if (x && typeof x === "object") Object.values(x).forEach(add);
  };
  add(rec.receiver);
  add(rec.args);
  add(rec.options?.relativeTo);
  if (rec.result !== null) add(rec.result);
  return out;
}

function firstCalendar(s) {
  const m = /\[!?u-ca=([^\]]*)\]/.exec(s);
  return m ? m[1].toLowerCase() : null;
}

function yearOutOfRange(rec) {
  for (const s of strings(rec)) {
    for (const m of s.matchAll(/(?:^|[^\d])([+-]\d{6})-\d\d-\d\d/g)) {
      if (Math.abs(parseInt(m[1], 10)) > 9999) return true;
    }
  }
  for (const a of rec.args ?? []) {
    if (a && typeof a === "object" && Math.abs(a.year ?? 0) > 9999) return true;
  }
  return false;
}

function epochNs(s) {
  for (const parse of [(x) => Temporal.Instant.from(x), (x) => Temporal.ZonedDateTime.from(x)]) {
    try {
      return parse(s).epochNanoseconds;
    } catch {
      // not this type
    }
  }
  return null;
}

function instantOutOfRange(type, rec) {
  const ns = [];
  if (type === "instant" || type === "zoned-date-time") {
    for (const s of strings(rec)) {
      const v = epochNs(s);
      if (v !== null) ns.push(v);
    }
  }
  if (rec.op === "fromEpochNanoseconds") ns.push(BigInt(rec.args[0]));
  if (rec.op === "fromEpochMilliseconds") ns.push(BigInt(rec.args[0]) * 1000000n);
  return ns.some((v) => v < INSTANT_MIN || v > INSTANT_MAX);
}

function durationOutOfRange(rec) {
  for (const s of strings(rec)) {
    if (!/^[+-]?P/i.test(s)) continue;
    let d;
    try {
      d = Temporal.Duration.from(s);
    } catch {
      continue;
    }
    for (const [unit, max] of Object.entries(SPAN_MAX)) {
      if (Math.abs(d[unit]) > max) return true;
    }
  }
  return false;
}

// The offset of a zoned value printed with seconds, as jiff does, in place of
// Temporal's offset rounded to minutes.
function fullOffset(type, rec) {
  if (rec.op !== "toString" || rec.result === null) return undefined;
  let zdt;
  if (type === "zoned-date-time") {
    zdt = Temporal.ZonedDateTime.from(rec.receiver);
  } else if (type === "instant" && rec.options?.timeZone) {
    zdt = Temporal.Instant.from(rec.receiver).toZonedDateTimeISO(rec.options.timeZone);
  } else {
    return undefined;
  }
  if (zdt.offsetNanoseconds % 60e9 === 0) return undefined;
  if (rec.options?.offset === "never") return undefined;
  const short = zdt.offset.slice(0, 6);
  const i = rec.result.lastIndexOf(short);
  return rec.result.slice(0, i) + zdt.offset + rec.result.slice(i + short.length);
}

export function divergence(type, rec, rerun) {
  const ok = rec.error === null;
  const strs = strings(rec);

  if (ok && CALENDAR_TYPES.has(type) && strs.some((s) => {
    const c = firstCalendar(s);
    return c !== null && c !== "iso8601";
  })) {
    return { divergence: "calendars", zeitig_error: "RangeError" };
  }
  if (ok && yearOutOfRange(rec)) {
    return { divergence: "date-range", zeitig_error: "RangeError" };
  }
  if (ok && instantOutOfRange(type, rec)) {
    return { divergence: "instant-range", zeitig_error: "RangeError" };
  }
  if (ok && durationOutOfRange(rec)) {
    return { divergence: "duration-range", zeitig_error: "RangeError" };
  }
  if (ok && (type === "instant" || type === "zoned-date-time") && rec.op === "from" &&
    /[+-]\d\d:?\d\d:?\d\d[.,]\d/.test(rec.receiver.split("[")[0])) {
    return { divergence: "offset-fractional-seconds", zeitig_error: "RangeError" };
  }
  // Without relativeTo, zeitig treats weeks as 7 days of 24 hours, which is
  // what Temporal gives relative to a plain date.
  if (!ok && type === "duration" && ["round", "total", "compare"].includes(rec.op) &&
    !rec.options?.relativeTo &&
    (strs.some((s) => /W/.test(s)) || Object.values(rec.options ?? {}).includes("week")) &&
    !strs.some((s) => /^-?P(\d+Y|\d+M)/.test(s))) {
    const r = rerun({ options: { ...rec.options, relativeTo: "2000-01-01" } });
    if (r.error === null) {
      return { divergence: "weeks-without-relative-to", zeitig_result: r.result };
    }
  }
  const offset = ok ? fullOffset(type, rec) : undefined;
  if (offset !== undefined) {
    return { divergence: "offset-seconds", zeitig_result: offset };
  }
  return undefined;
}
