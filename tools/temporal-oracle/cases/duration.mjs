import { rng, ROUNDING_MODES, ALL_UNITS, TIME_UNITS, INCREMENTS, opts } from "./lib.mjs";

const r = rng(4);

const TIME_DURS = ["PT0S", "PT1H", "PT130M", "PT36H", "-PT36H", "PT1.5S", "PT0.000000001S",
  "P1D", "P2DT12H", "-P1DT0.5S", "PT90M30.25S", "PT1000000.123456789S", "P3DT4H5M6.007008009S",
  "PT59.999999999S"];
const CAL_DURS = ["P1Y", "P1M", "P1W", "P1Y2M3W4D", "-P1M15D", "P1M1DT12H", "P13M", "P45D",
  "P2W3D", "P1Y6M", "-P2Y", "P10W"];
const RELATIVE = ["2020-01-01", "2020-01-31", "2019-02-28T12:00", "2021-03-14T01:00-08:00[America/Los_Angeles]",
  "2020-10-25T00:00+01:00[Europe/London]"];

export default function cases() {
  const out = [];

  const parse = [
    ...TIME_DURS, ...CAL_DURS, "P", "PT", "P1", "PT1.5H", "PT1.5M", "P1.5D", "P1.5Y",
    "PT1.123456789S", "PT1.1234567891S", "p1d", "P1DT", "-P1D", "+P1D", "−P1D", "P1Y1Y",
    "PT1H1.5M", "PT1.5H1M", "P1W1D", "PT0.5H", "P0D", "P1M-1D", "PT1,5S", "P19999Y", "P20000Y",
    "PT9007199254740991S", "PT9007199254740992S", "P1000000D", "PT100000000H",
  ];
  for (const s of parse) out.push({ op: "from", receiver: s });

  for (const f of [{ years: 1, months: -1 }, { hours: 1.5 }, { days: 1, hours: 25 },
    { years: 0 }, { weeks: 2, nanoseconds: 1 }]) {
    out.push({ op: "from_fields", args: [f] });
  }

  for (const d of [...TIME_DURS, ...CAL_DURS]) {
    out.push({ op: "negated", receiver: d });
    out.push({ op: "abs", receiver: d });
    out.push({ op: "get", receiver: d, args: ["sign"] });
    out.push({ op: "get", receiver: d, args: ["blank"] });
  }

  for (let k = 0; k < 30; k++) {
    out.push({ op: "add", receiver: r.pick(TIME_DURS), args: [r.pick(TIME_DURS)] });
    out.push({ op: "subtract", receiver: r.pick(TIME_DURS), args: [r.pick(TIME_DURS)] });
  }
  out.push({ op: "add", receiver: "P1M", args: ["P1D"] });

  // round() without relativeTo (time units and days).
  for (const d of TIME_DURS) {
    for (const largestUnit of [undefined, "day", "hour", "minute", "second"]) {
      for (const smallestUnit of [undefined, "hour", "millisecond"]) {
        if (!largestUnit && !smallestUnit) continue;
        out.push({ op: "round", receiver: d, options: opts({ largestUnit, smallestUnit, roundingMode: r.pick(ROUNDING_MODES) }) });
      }
    }
  }
  for (const smallestUnit of TIME_UNITS) {
    for (const roundingMode of ROUNDING_MODES) {
      out.push({ op: "round", receiver: r.pick(TIME_DURS), options: { smallestUnit, roundingMode, roundingIncrement: r.pick(INCREMENTS[smallestUnit]) } });
    }
  }
  out.push({ op: "round", receiver: "P1D", options: { smallestUnit: "day", roundingIncrement: 2 } });
  out.push({ op: "round", receiver: "P1M", options: { smallestUnit: "day" } });
  out.push({ op: "round", receiver: "P1W", options: { largestUnit: "day" } });
  out.push({ op: "round", receiver: "P10D", options: { largestUnit: "week" } });

  // round() with relativeTo.
  for (const relativeTo of RELATIVE) {
    for (const d of [...CAL_DURS, "P2DT12H", "PT36H", "-PT36H"].filter(() => r.next() < 0.5)) {
      for (const [largestUnit, smallestUnit] of [["year", "day"], ["month", "day"], ["week", "day"],
        ["day", "hour"], [undefined, "month"], ["year", "year"], ["month", "week"], [undefined, "day"]]) {
        out.push({ op: "round", receiver: d, options: opts({ largestUnit, smallestUnit, roundingMode: r.pick(ROUNDING_MODES), relativeTo }) });
      }
    }
  }

  // total().
  for (const d of TIME_DURS) {
    for (const unit of ["day", "hour", "minute", "second", "millisecond", "nanosecond"]) {
      out.push({ op: "total", receiver: d, options: { unit } });
    }
  }
  for (const relativeTo of RELATIVE) {
    for (const d of CAL_DURS.filter(() => r.next() < 0.5)) {
      for (const unit of ["year", "month", "week", "day", "hour"]) {
        out.push({ op: "total", receiver: d, options: { unit, relativeTo } });
      }
    }
  }
  out.push({ op: "total", receiver: "P1M", options: { unit: "day" } });
  out.push({ op: "total", receiver: "P2W", options: { unit: "hour" } });

  // compare().
  for (let k = 0; k < 25; k++) {
    out.push({ op: "compare", receiver: r.pick(TIME_DURS), args: [r.pick(TIME_DURS)] });
    out.push({ op: "compare", receiver: r.pick(CAL_DURS), args: [r.pick(CAL_DURS)], options: { relativeTo: r.pick(RELATIVE) } });
  }
  out.push({ op: "compare", receiver: "P1M", args: ["P30D"] });
  out.push({ op: "compare", receiver: "P1W", args: ["P7D"] });

  // toString().
  for (const d of [...TIME_DURS, "P1Y2M3W4DT5H6M7.123456789S"]) {
    out.push({ op: "toString", receiver: d });
    for (const fractionalSecondDigits of [0, 3, 9]) {
      out.push({ op: "toString", receiver: d, options: { fractionalSecondDigits } });
    }
    out.push({ op: "toString", receiver: d, options: { smallestUnit: "millisecond", roundingMode: "halfExpand" } });
  }
  return out;
}
