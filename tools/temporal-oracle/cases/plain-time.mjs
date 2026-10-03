import { rng, ROUNDING_MODES, TIME_UNITS, INCREMENTS, unitPairs, opts } from "./lib.mjs";

const r = rng(2);

const TIMES = [
  "00:00", "12:00", "23:59:59.999999999", "19:39:09.068346205", "03:24:30.0000035",
  "12:30:45.5", "00:00:00.000000001", "11:59:59.5", "13:45", "06:07:08.009010011",
];

export default function cases() {
  const out = [];

  const parse = [
    ...TIMES, "T12:00", "t12:00", "1200", "120000", "12", "T12", "12:00:00.123456789123",
    "12:00:60", "24:00", "25:00", "12:60", "12:00Z", "12:00+01:00", "12:00[Europe/Paris]",
    "12:00[u-ca=iso8601]", "12:00[u-ca=hebrew]", "2020-01-01T12:00", "2020-01-01",
    "2020-01-01T12:00Z", "12:00:00,5", "12:00:00.", "12:00-12", "12:00:00+24:00",
    "1970-01-01T12:00[UTC]", "12:00 ", "",
  ];
  for (const s of parse) out.push({ op: "from", receiver: s });

  for (const overflow of ["constrain", "reject"]) {
    for (const f of [
      { hour: 24 }, { hour: 23, minute: 60 }, { second: 61 }, { millisecond: 1000 },
      { microsecond: 1000 }, { nanosecond: 1000 }, { hour: -1 }, { hour: 12, minute: 30 },
      { hour: 1, minute: 2, second: 3, millisecond: 4, microsecond: 5, nanosecond: 6 },
    ]) {
      out.push({ op: "from_fields", args: [f], options: { overflow } });
    }
  }

  const durations = ["PT1H", "-PT1H", "PT25H", "P1D", "-P1D", "P1Y", "P1M", "P1W",
    "PT1.5S", "PT0.000000001S", "-PT0.000000001S", "PT12H30M", "PT86400S", "PT1440M",
    "PT36H59M59.999999999S", "-PT100H", "P1DT1H"];
  for (const op of ["add", "subtract"]) {
    for (const t of ["23:30", "19:39:09.068346205"]) {
      for (const d of durations) out.push({ op, receiver: t, args: [d] });
    }
  }

  for (const op of ["until", "since"]) {
    for (const [largestUnit, smallestUnit] of unitPairs(TIME_UNITS)) {
      for (const roundingMode of ROUNDING_MODES) {
        const roundingIncrement = r.next() < 0.5 ? 1 : r.pick(INCREMENTS[smallestUnit]);
        out.push({
          op, receiver: r.pick(TIMES), args: [r.pick(TIMES)],
          options: opts({ largestUnit, smallestUnit, roundingMode, roundingIncrement }),
        });
      }
    }
    for (const [a, b] of [["00:00", "23:59:59.999999999"], ["19:39:09.068346205", "03:24:30"]]) {
      out.push({ op, receiver: a, args: [b] });
      out.push({ op, receiver: a, args: [b], options: { largestUnit: "day" } });
      out.push({ op, receiver: a, args: [b], options: { smallestUnit: "day" } });
      out.push({ op, receiver: a, args: [b], options: { smallestUnit: "hour", roundingIncrement: 5 } });
      out.push({ op, receiver: a, args: [b], options: { smallestUnit: "minute", roundingIncrement: 24 } });
    }
  }

  for (const smallestUnit of TIME_UNITS) {
    for (const roundingMode of ROUNDING_MODES) {
      for (const t of TIMES.slice(0, 3)) {
        out.push({ op: "round", receiver: t, options: { smallestUnit, roundingMode } });
      }
      const inc = r.pick(INCREMENTS[smallestUnit]);
      out.push({ op: "round", receiver: r.pick(TIMES), options: { smallestUnit, roundingMode, roundingIncrement: inc } });
    }
    out.push({ op: "round", receiver: "12:00", options: { smallestUnit, roundingIncrement: 7 } });
  }
  out.push({ op: "round", receiver: "12:00", options: { smallestUnit: "day" } });
  out.push({ op: "round", receiver: "12:00", options: { smallestUnit: "hour", roundingIncrement: 24 } });

  for (const f of [{ hour: 3 }, { minute: 60 }, { second: 59, nanosecond: 999 },
    { hour: 24 }, { millisecond: 999, microsecond: 999 }]) {
    for (const overflow of [undefined, "reject"]) {
      out.push({ op: "with", receiver: "19:39:09.068346205", args: [f], options: opts({ overflow }) });
    }
  }

  for (const t of TIMES) {
    out.push({ op: "toString", receiver: t });
    for (const fractionalSecondDigits of [0, 1, 5, 9]) {
      out.push({ op: "toString", receiver: t, options: { fractionalSecondDigits } });
    }
    for (const smallestUnit of ["minute", "second", "millisecond", "microsecond", "nanosecond"]) {
      out.push({ op: "toString", receiver: t, options: { smallestUnit } });
    }
    for (const roundingMode of ROUNDING_MODES.filter(() => r.next() < 0.4)) {
      out.push({ op: "toString", receiver: t, options: { fractionalSecondDigits: r.int(0, 8), roundingMode } });
    }
  }
  out.push({ op: "toString", receiver: "12:00", options: { smallestUnit: "hour" } });
  out.push({ op: "toString", receiver: "12:00", options: { fractionalSecondDigits: 10 } });

  for (let k = 0; k < 12; k++) {
    const a = r.pick(TIMES);
    const b = k % 4 === 0 ? a : r.pick(TIMES);
    out.push({ op: "compare", receiver: a, args: [b] });
    out.push({ op: "equals", receiver: a, args: [b] });
  }
  for (const g of ["hour", "minute", "second", "millisecond", "microsecond", "nanosecond"]) {
    out.push({ op: "get", receiver: "19:39:09.068346205", args: [g] });
  }
  return out;
}
