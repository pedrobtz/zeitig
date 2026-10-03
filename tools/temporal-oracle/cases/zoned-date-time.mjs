import { rng, ROUNDING_MODES, ALL_UNITS, INCREMENTS, unitPairs, opts } from "./lib.mjs";

const r = rng(6);

// Dates are kept between 2000 and 2024, where the IANA rules for these zones
// are settled, so the fixtures do not depend on the tzdata release.
const ZDTS = [
  "2019-03-10T01:30-05:00[America/New_York]", "2019-11-03T01:30-04:00[America/New_York]",
  "2019-11-03T01:30-05:00[America/New_York]", "2019-03-31T00:30+00:00[Europe/London]",
  "2019-10-27T01:30+01:00[Europe/London]", "2019-04-07T01:45+11:00[Australia/Lord_Howe]",
  "2019-10-06T01:45+10:30[Australia/Lord_Howe]", "2011-12-29T12:00-10:00[Pacific/Apia]",
  "2020-06-01T12:00:00.123456789+05:30[Asia/Kolkata]", "2020-01-01T00:00+00:00[UTC]",
  "2020-02-29T23:59:59.999999999+05:30[+05:30]", "2015-10-17T23:30-03:00[America/Sao_Paulo]",
];

export default function cases() {
  const out = [];

  const parse = [
    ...ZDTS, "2020-01-01T12:00[Europe/Paris]", "2020-01-01T12:00Z[Europe/Paris]",
    "2020-01-01T12:00+01:00[Europe/Paris]", "2020-01-01T12:00+02:00[Europe/Paris]",
    "2020-01-01T12:00[!Europe/Paris]", "2020-01-01T12:00+01:00[europe/paris]",
    "2020-01-01T12:00[Etc/GMT+5]",
    "2020-01-01T12:00[+01:00]", "2020-01-01T12:00+01:00[+01:00]", "2020-01-01T12:00[+01]",
    "2020-01-01T12:00[+01:00:30]", "2020-01-01T12:00[Mars/Olympus]", "2020-01-01T12:00",
    "2020-01-01T12:00+01:00", "2020-01-01T12:00[Europe/Paris][u-ca=iso8601]",
    "2020-01-01T12:00[Europe/Paris][u-ca=hebrew]", "2020-01-01T12:00[UTC]", "2020-01-01T12:00[utc]",
    "2020-01-01T12:00[Etc/UTC]",
    "2019-03-10T02:30[America/New_York]", "2019-11-03T01:30[America/New_York]",
    "2019-03-10T02:30-05:00[America/New_York]", "2019-03-10T02:30-04:00[America/New_York]",
    "1900-01-01T00:00+00:01[Europe/London]",
  ];
  for (const s of parse) out.push({ op: "from", receiver: s });

  // Every disambiguation x offset option on gaps, overlaps and mismatched offsets.
  const ambiguous = [
    "2019-03-10T02:30-05:00[America/New_York]", "2019-03-10T02:30-04:00[America/New_York]",
    "2019-11-03T01:30-04:00[America/New_York]", "2019-11-03T01:30-05:00[America/New_York]",
    "2019-11-03T01:30-06:00[America/New_York]", "2019-10-06T02:15+10:30[Australia/Lord_Howe]",
    "2019-04-07T01:45+10:30[Australia/Lord_Howe]", "2019-04-07T01:45+11:00[Australia/Lord_Howe]",
    "2011-12-30T12:00-10:00[Pacific/Apia]", "2020-06-01T12:00+01:00[Europe/London]",
    "2019-03-10T02:30[America/New_York]",
  ];
  for (const s of ambiguous) {
    for (const disambiguation of ["compatible", "earlier", "later", "reject"]) {
      for (const offset of ["use", "prefer", "ignore", "reject"]) {
        out.push({ op: "from", receiver: s, options: { disambiguation, offset } });
      }
    }
  }

  for (const overflow of ["constrain", "reject"]) {
    for (const f of [
      { year: 2019, month: 3, day: 10, hour: 2, minute: 30, timeZone: "America/New_York" },
      { year: 2019, month: 2, day: 30, timeZone: "Europe/London" },
      { year: 2011, month: 12, day: 30, hour: 1, timeZone: "Pacific/Apia" },
    ]) {
      out.push({ op: "from_fields", args: [f], options: { overflow } });
    }
  }

  const durations = ["P1D", "-P1D", "PT24H", "-PT24H", "PT30M", "P1M", "P1DT1H",
    "-P1M1DT0.000000001S"];
  for (const op of ["add", "subtract"]) {
    for (const z of ZDTS) {
      for (const d of durations) out.push({ op, receiver: z, args: [d] });
    }
    out.push({ op, receiver: "2020-01-31T12:00+00:00[UTC]", args: ["P1M"], options: { overflow: "reject" } });
  }

  const pairs = [
    ["2019-03-09T12:00-05:00[America/New_York]", "2019-03-11T12:00-04:00[America/New_York]"],
    ["2019-11-02T12:00-04:00[America/New_York]", "2019-11-04T00:00-05:00[America/New_York]"],
    ["2019-03-10T01:30-05:00[America/New_York]", "2019-03-10T03:30-04:00[America/New_York]"],
    ["2019-10-05T12:00+10:30[Australia/Lord_Howe]", "2019-10-07T12:00+11:00[Australia/Lord_Howe]"],
    ["2011-12-29T12:00-10:00[Pacific/Apia]", "2011-12-31T12:00+14:00[Pacific/Apia]"],
    ["2020-01-31T00:00+00:00[Europe/London]", "2020-03-31T00:00+01:00[Europe/London]"],
    ["2020-01-01T00:00+05:30[Asia/Kolkata]", "2021-06-15T12:34:56.789+05:30[Asia/Kolkata]"],
    ["2019-11-03T01:30-04:00[America/New_York]", "2019-11-03T01:30-05:00[America/New_York]"],
  ];
  for (const op of ["until", "since"]) {
    for (const [largestUnit, smallestUnit] of unitPairs(ALL_UNITS)) {
      const [a, b] = r.pick(pairs);
      const swap = r.next() < 0.5;
      const inc = INCREMENTS[smallestUnit] ? r.pick(INCREMENTS[smallestUnit]) : 1;
      out.push({
        op, receiver: swap ? b : a, args: [swap ? a : b],
        options: opts({ largestUnit, smallestUnit, roundingMode: r.pick(ROUNDING_MODES), roundingIncrement: r.next() < 0.5 ? 1 : inc }),
      });
    }
    for (const [a, b] of pairs) {
      out.push({ op, receiver: a, args: [b] });
      for (const largestUnit of ["year", "month", "week", "day"]) {
        out.push({ op, receiver: a, args: [b], options: { largestUnit } });
      }
    }
    out.push({ op, receiver: ZDTS[0], args: ["2019-03-10T01:30-05:00[America/Chicago]"] });
    out.push({ op, receiver: ZDTS[0], args: ["2019-03-12T01:30-05:00[America/Chicago]"], options: { largestUnit: "day" } });
  }

  for (const smallestUnit of ["day", "hour", "minute", "second", "millisecond", "microsecond", "nanosecond"]) {
    for (const roundingMode of ROUNDING_MODES) {
      out.push({ op: "round", receiver: r.pick(ZDTS), options: { smallestUnit, roundingMode } });
    }
    for (const z of ZDTS) {
      out.push({ op: "round", receiver: z, options: { smallestUnit, roundingMode: "halfExpand" } });
    }
  }
  out.push({ op: "round", receiver: ZDTS[0], options: { smallestUnit: "minute", roundingIncrement: 15 } });
  out.push({ op: "round", receiver: ZDTS[0], options: { smallestUnit: "day", roundingIncrement: 2 } });

  for (const z of ZDTS) {
    for (const f of [{ hour: 2, minute: 30 }, { minute: 45 }, { day: 31 }, { month: 2, day: 30 }]) {
      out.push({ op: "with", receiver: z, args: [f] });
    }
  }
  for (const f of [{ minute: 45 }, { hour: 2 }]) {
    for (const disambiguation of ["compatible", "earlier", "later", "reject"]) {
      for (const offset of ["use", "prefer", "ignore", "reject"]) {
        out.push({ op: "with", receiver: "2019-11-03T01:30-05:00[America/New_York]", args: [f], options: { disambiguation, offset } });
        out.push({ op: "with", receiver: "2019-03-10T01:30-05:00[America/New_York]", args: [f], options: { disambiguation, offset } });
      }
    }
  }

  for (const z of ZDTS) {
    out.push({ op: "toString", receiver: z });
    out.push({ op: "toString", receiver: z, options: { fractionalSecondDigits: 3, roundingMode: r.pick(ROUNDING_MODES) } });
    out.push({ op: "toString", receiver: z, options: { smallestUnit: "minute" } });
    out.push({ op: "toString", receiver: z, options: { offset: "never" } });
    out.push({ op: "toString", receiver: z, options: { timeZoneName: "never" } });
    out.push({ op: "toString", receiver: z, options: { timeZoneName: "critical", calendarName: "always" } });
    for (const g of ["offset", "offsetNanoseconds", "hoursInDay", "timeZoneId", "epochNanoseconds",
      "dayOfWeek", "hour"]) {
      out.push({ op: "get", receiver: z, args: [g] });
    }
    out.push({ op: "startOfDay", receiver: z });
    out.push({ op: "getTimeZoneTransition", receiver: z, args: ["previous"] });
    out.push({ op: "withTimeZone", receiver: z, args: ["Asia/Tokyo"] });
    out.push({ op: "toInstant", receiver: z });
    out.push({ op: "toPlainDateTime", receiver: z });
  }
  for (const z of ["2019-01-01T00:00+00:00[Europe/London]", "2019-01-01T00:00-05:00[America/New_York]",
    "2019-01-01T00:00+11:00[Australia/Lord_Howe]", "2020-01-01T00:00+00:00[UTC]"]) {
    out.push({ op: "getTimeZoneTransition", receiver: z, args: ["next"] });
  }
  for (const [z, tz] of [["2011-12-29T00:00-10:00[Pacific/Apia]", "Pacific/Apia"],
    ["2015-10-18T12:00-02:00[America/Sao_Paulo]", "America/Sao_Paulo"]]) {
    out.push({ op: "startOfDay", receiver: z });
    out.push({ op: "get", receiver: z, args: ["hoursInDay"] });
  }

  for (let k = 0; k < 12; k++) {
    const a = r.pick(ZDTS);
    const b = k % 4 === 0 ? a : r.pick(ZDTS);
    out.push({ op: "compare", receiver: a, args: [b] });
    out.push({ op: "equals", receiver: a, args: [b] });
  }
  out.push({ op: "equals", receiver: "2020-01-01T00:00+00:00[UTC]", args: ["2020-01-01T00:00+00:00[Etc/UTC]"] });
  out.push({ op: "equals", receiver: "2020-01-01T00:00+00:00[UTC]", args: ["2020-01-01T00:00+00:00[+00:00]"] });
  return out;
}
