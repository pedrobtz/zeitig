import { rng, ROUNDING_MODES, TIME_UNITS, INCREMENTS, unitPairs, opts } from "./lib.mjs";

const r = rng(5);

const INSTANTS = ["1970-01-01T00:00Z", "2019-03-30T00:45:00.123456789Z", "1969-07-20T20:17Z",
  "2020-02-29T23:59:59.999999999Z", "1900-01-01T12:00:00.5Z", "2038-01-19T03:14:07Z",
  "2001-09-09T01:46:40.000000001Z", "1969-12-31T23:59:59.999999999Z"];

export default function cases() {
  const out = [];

  const parse = [
    ...INSTANTS, "2020-01-01T12:00+01:00", "2020-01-01T12:00:00.5-05:30", "2020-01-01T12:00",
    "2020-01-01", "2020-01-01T12:00Z[Europe/Paris]", "2020-01-01T12:00+01:00[Asia/Tokyo]",
    "2020-01-01T12:00Z[u-ca=hebrew]", "2020-01-01T12:00z", "2020-01-01T24:00Z",
    "2020-01-01T23:59:60Z", "20200101T120000Z", "2020-01-01T12Z", "2020-01-01T12:00+0100",
    "2020-01-01T12:00+01", "2020-01-01T12:00+01:00:30", "2020-01-01T12:00+01:00:30.123456789",
    "-271821-04-20T00:00Z", "+275760-09-13T00:00Z", "+275760-09-13T00:00:00.000000001Z",
    "-009999-01-01T00:00Z", "9999-12-31T23:59:59.999999999Z", "+010000-01-01T00:00Z",
    "-009999-01-01T00:00+01:00", "9999-12-31T23:59:59.999999999-01:00", "2020-01-01T12:00+25:00",
  ];
  for (const s of parse) out.push({ op: "from", receiver: s });

  for (const ns of ["0", "1", "-1", "1553906700000000001", "-1553906700000000001",
    "8640000000000000000000", "8640000000000000000001", "253402300799999999999", "253402300800000000000",
    "-377705116800000000000", "-377705116800000000001"]) {
    out.push({ op: "fromEpochNanoseconds", args: [ns] });
  }
  for (const ms of [0, 1553906700000, -1, 253402300799999, 253402300800000]) {
    out.push({ op: "fromEpochMilliseconds", args: [ms] });
  }

  const durations = ["PT1H", "-PT1H", "PT1.000000001S", "PT24H", "P1D", "PT8760H", "P1M", "P1Y",
    "P1W", "-PT0.000000001S", "PT2562047H47M16.854775807S"];
  for (const op of ["add", "subtract"]) {
    for (const i of INSTANTS.slice(0, 4)) {
      for (const d of durations) out.push({ op, receiver: i, args: [d] });
    }
  }

  for (const op of ["until", "since"]) {
    for (const [largestUnit, smallestUnit] of unitPairs(TIME_UNITS)) {
      for (const roundingMode of ROUNDING_MODES.filter(() => r.next() < 0.5)) {
        out.push({
          op, receiver: r.pick(INSTANTS), args: [r.pick(INSTANTS)],
          options: opts({ largestUnit, smallestUnit, roundingMode, roundingIncrement: r.next() < 0.5 ? 1 : r.pick(INCREMENTS[smallestUnit]) }),
        });
      }
    }
    for (const largestUnit of ["day", "week", "month", "year"]) {
      out.push({ op, receiver: INSTANTS[0], args: [INSTANTS[1]], options: { largestUnit } });
    }
    out.push({ op, receiver: INSTANTS[0], args: [INSTANTS[1]] });
  }

  const instantIncrements = {
    hour: [1, 2, 3, 4, 6, 8, 12, 24], minute: [1, 15, 30, 60, 90, 1440], second: [1, 30, 86400],
    millisecond: [1, 10, 86400000], microsecond: [1, 1000], nanosecond: [1, 7],
  };
  for (const smallestUnit of TIME_UNITS) {
    for (const roundingMode of ROUNDING_MODES) {
      for (const roundingIncrement of instantIncrements[smallestUnit].filter(() => r.next() < 0.6)) {
        out.push({ op: "round", receiver: r.pick(INSTANTS), options: { smallestUnit, roundingMode, roundingIncrement } });
      }
    }
  }
  out.push({ op: "round", receiver: INSTANTS[1], options: { smallestUnit: "day" } });
  out.push({ op: "round", receiver: INSTANTS[1], options: { smallestUnit: "hour", roundingIncrement: 5 } });

  for (const i of INSTANTS) {
    out.push({ op: "toString", receiver: i });
    for (const fractionalSecondDigits of [0, 4, 9]) {
      out.push({ op: "toString", receiver: i, options: { fractionalSecondDigits, roundingMode: r.pick(ROUNDING_MODES) } });
    }
    out.push({ op: "toString", receiver: i, options: { smallestUnit: "minute" } });
    for (const timeZone of ["UTC", "Asia/Kolkata", "America/New_York", "+01:00", "Europe/Dublin"]) {
      out.push({ op: "toString", receiver: i, options: { timeZone } });
    }
    for (const tz of ["UTC", "Asia/Tokyo", "Australia/Lord_Howe", "-03:30"]) {
      out.push({ op: "toZonedDateTimeISO", receiver: i, args: [tz] });
    }
    out.push({ op: "get", receiver: i, args: ["epochMilliseconds"] });
    out.push({ op: "get", receiver: i, args: ["epochNanoseconds"] });
  }

  for (let k = 0; k < 12; k++) {
    const a = r.pick(INSTANTS);
    const b = k % 4 === 0 ? a : r.pick(INSTANTS);
    out.push({ op: "compare", receiver: a, args: [b] });
    out.push({ op: "equals", receiver: a, args: [b] });
  }
  return out;
}
