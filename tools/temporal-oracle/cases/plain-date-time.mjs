import { rng, ROUNDING_MODES, ALL_UNITS, INCREMENTS, unitPairs, opts } from "./lib.mjs";

const r = rng(3);

const DTS = [
  "2020-01-01T00:00", "2020-02-29T23:59:59.999999999", "2021-01-31T12:00",
  "1995-12-07T03:24:30.0000035", "2019-03-31T01:30", "2016-12-31T23:59:59",
  "2024-07-15T16:24:59.123", "2000-01-01T00:00:00.000000001", "2010-10-10T10:10:10.10101",
  "1999-12-31T18:00",
];

export default function cases() {
  const out = [];

  const parse = [
    ...DTS, "2020-01-01", "2020-01-01T24:00", "2020-01-01T12:00Z", "2020-01-01T12:00+01:00",
    "2020-01-01T12:00:00+01:00[Europe/Paris]", "2020-01-01T12:00[u-ca=iso8601]",
    "2020-01-01T12:00[!u-ca=iso8601]", "2020-01-01T12:00[u-ca=hebrew]",
    "2020-01-01T12:00[u-ca=hebrew][!u-ca=iso8601]", "20200101T120000.5", "2020-01-01T12",
    "2020-01-01T12:00:60", "2020-01-01T12:00:00.1234567890", "+010000-01-01T00:00",
    "-271821-04-19T00:00:00.000000001", "-271821-04-19T00:00", "+275760-09-13T00:00",
    "2020-01-01T12:00-24:00", "2020-01-01T12:00+00:00:00.5",
  ];
  for (const s of parse) out.push({ op: "from", receiver: s });

  for (const overflow of ["constrain", "reject"]) {
    for (const f of [
      { year: 2021, month: 2, day: 29, hour: 24 }, { year: 2020, month: 1, day: 1, minute: 61 },
      { year: 2020, month: 13, day: 40, second: 60 }, { year: 2020, month: 6, day: 1, hour: 12 },
    ]) {
      out.push({ op: "from_fields", args: [f], options: { overflow } });
    }
  }

  const durations = ["P1M", "-P1M", "P1Y", "PT24H", "PT25H", "-PT1H", "P1DT1H", "P1M1DT1H1M1S",
    "PT0.000000001S", "-PT0.000000001S", "P1W", "PT48H30M", "-P1Y2M3DT4H5M6.789S", "PT1000000H"];
  for (const op of ["add", "subtract"]) {
    for (const dt of DTS.slice(0, 3)) {
      for (const d of durations) out.push({ op, receiver: dt, args: [d] });
    }
    out.push({ op, receiver: "2021-01-31T12:00", args: ["P1M"], options: { overflow: "reject" } });
  }

  for (const op of ["until", "since"]) {
    for (const [largestUnit, smallestUnit] of unitPairs(ALL_UNITS)) {
      for (const roundingMode of [r.pick(ROUNDING_MODES), r.pick(ROUNDING_MODES)]) {
        const inc = INCREMENTS[smallestUnit] ? r.pick(INCREMENTS[smallestUnit]) : r.pick([1, 2, 3]);
        out.push({
          op, receiver: r.pick(DTS), args: [r.pick(DTS)],
          options: opts({ largestUnit, smallestUnit, roundingMode, roundingIncrement: r.next() < 0.5 ? 1 : inc }),
        });
      }
    }
    for (const [a, b] of [["2020-01-01T00:00", "2021-03-04T05:06:07.008009010"],
      ["2021-03-04T05:06:07.008009010", "2020-01-01T00:00"], ["2020-02-29T23:00", "2021-02-28T01:00"]]) {
      out.push({ op, receiver: a, args: [b] });
      for (const largestUnit of ["year", "month", "week", "day", "hour", "second", "nanosecond"]) {
        out.push({ op, receiver: a, args: [b], options: { largestUnit } });
      }
    }
  }

  for (const smallestUnit of ["day", "hour", "minute", "second", "millisecond", "microsecond", "nanosecond"]) {
    for (const roundingMode of ROUNDING_MODES) {
      for (const dt of DTS.slice(1, 3)) {
        out.push({ op: "round", receiver: dt, options: { smallestUnit, roundingMode } });
      }
      const inc = smallestUnit === "day" ? 1 : r.pick(INCREMENTS[smallestUnit]);
      out.push({ op: "round", receiver: r.pick(DTS), options: { smallestUnit, roundingMode, roundingIncrement: inc } });
    }
  }
  out.push({ op: "round", receiver: "2020-01-01T12:00", options: { smallestUnit: "day", roundingIncrement: 2 } });
  out.push({ op: "round", receiver: "2020-01-01T12:00", options: { smallestUnit: "month" } });
  out.push({ op: "round", receiver: "9999-12-31T23:59:59.5", options: { smallestUnit: "second" } });

  for (const f of [{ hour: 12 }, { day: 31 }, { month: 2, day: 30 }, { year: 2021, minute: 99 },
    { nanosecond: 1 }]) {
    for (const overflow of [undefined, "reject"]) {
      out.push({ op: "with", receiver: "2020-01-31T19:39:09.068346205", args: [f], options: opts({ overflow }) });
    }
  }

  for (const dt of DTS) {
    out.push({ op: "toString", receiver: dt });
    for (const fractionalSecondDigits of [0, 2, 9]) {
      out.push({ op: "toString", receiver: dt, options: { fractionalSecondDigits } });
    }
    for (const smallestUnit of ["minute", "second", "microsecond"]) {
      out.push({ op: "toString", receiver: dt, options: { smallestUnit, roundingMode: r.pick(ROUNDING_MODES) } });
    }
    for (const calendarName of ["always", "never", "critical"]) {
      out.push({ op: "toString", receiver: dt, options: { calendarName } });
    }
  }
  out.push({ op: "toString", receiver: "9999-12-31T23:59:59.9", options: { smallestUnit: "second", roundingMode: "ceil" } });

  for (let k = 0; k < 12; k++) {
    const a = r.pick(DTS);
    const b = k % 4 === 0 ? a : r.pick(DTS);
    out.push({ op: "compare", receiver: a, args: [b] });
    out.push({ op: "equals", receiver: a, args: [b] });
  }

  // Wall-clock -> zoned, with every disambiguation in gaps and overlaps.
  const zoned = [
    ["2019-03-10T02:30", "America/New_York"], ["2019-11-03T01:30", "America/New_York"],
    ["2019-03-31T01:30", "Europe/London"], ["2019-10-27T01:30", "Europe/London"],
    ["2019-04-07T01:45", "Australia/Lord_Howe"], ["2019-10-06T02:15", "Australia/Lord_Howe"],
    ["2011-12-30T12:00", "Pacific/Apia"], ["2011-12-29T23:59", "Pacific/Apia"],
    ["2020-06-01T12:00", "Asia/Kolkata"], ["2020-06-01T12:00", "+05:30"],
    ["2020-06-01T12:00", "UTC"], ["2015-10-18T00:30", "America/Sao_Paulo"],
  ];
  for (const [dt, tz] of zoned) {
    for (const disambiguation of [undefined, "compatible", "earlier", "later", "reject"]) {
      out.push({ op: "toZonedDateTime", receiver: dt, args: [tz], options: opts({ disambiguation }) });
    }
  }

  for (const g of ["year", "month", "day", "hour", "minute", "second", "millisecond",
    "microsecond", "nanosecond", "dayOfWeek", "dayOfYear", "weekOfYear", "inLeapYear"]) {
    out.push({ op: "get", receiver: "2020-02-29T19:39:09.068346205", args: [g] });
  }
  return out;
}
