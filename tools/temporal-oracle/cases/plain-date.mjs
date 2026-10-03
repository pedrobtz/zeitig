import { rng, ROUNDING_MODES, DATE_UNITS, unitPairs, opts } from "./lib.mjs";

const r = rng(1);

const DATES = [
  "2020-01-01", "2020-02-29", "2021-01-31", "2021-02-28", "2019-12-31", "2000-02-29",
  "1900-03-01", "1970-01-01", "2024-03-15", "2023-08-31", "1999-05-31", "2100-02-28",
  "0001-01-01", "-000001-12-31", "9999-12-31", "-009999-01-01",
];

export default function cases() {
  const out = [];

  // Parsing.
  const parse = [
    ...DATES,
    "20200101", "2020-W01-1", "2020-001", "2020-01", "--01-01", "2020-01-01T12:00",
    "2020-01-01T23:59:59.999999999", "2020-01-01T12:00+01:00", "2020-01-01T12:00Z",
    "2020-01-01[Europe/Paris]", "2020-01-01T00:00[!Europe/Paris]", "2020-01-01[u-ca=iso8601]",
    "2020-01-01[!u-ca=iso8601]", "2024-03-15[u-ca=hebrew]", "2024-03-15[u-ca=gregory]",
    "2024-03-15[!u-ca=japanese]", "2020-01-01[u-ca=iso8601][u-ca=hebrew]",
    "2020-01-01[foo=bar]", "2020-01-01[!foo=bar]", "2020-13-01", "2020-02-30", "2021-02-29",
    "2020-00-10", "2020-1-1", "+002020-01-01", "-000000-01-01", "+010000-01-01",
    "-010000-01-01", "+275760-09-13", "2020-01-01T25:00", "2020-01-01T24:00",
    "2020-01-01t12:00", "2020-01-01 12:00", "2020-01-01T12:00:60", "", "garbage",
    " 2020-01-01", "2020-01-01T12:00-00:00", "2020-01-01T12:00+23:59",
    "2020-01-01T12:00+24:00", "2020-01-01T12:00:00,5",
  ];
  for (const s of parse) out.push({ op: "from", receiver: s });

  // Construction from fields.
  for (const overflow of ["constrain", "reject"]) {
    for (const [y, m, d] of [
      [2021, 2, 31], [2020, 2, 30], [2020, 13, 1], [2020, 12, 32], [2020, 0, 1],
      [2020, 1, 0], [2020, -1, 1], [2020, 4, 31], [2019, 2, 29], [9999, 12, 31],
      [10000, 1, 1], [-9999, 1, 1], [-10000, 12, 31], [2020, 6, 15],
    ]) {
      out.push({ op: "from_fields", args: [{ year: y, month: m, day: d }], options: { overflow } });
    }
  }

  // Arithmetic.
  const durations = [
    "P1M", "-P1M", "P1Y", "-P1Y", "P1D", "-P1D", "P1W", "P1Y1M1D", "-P1Y1M1D", "P13M",
    "P400D", "PT24H", "PT23H59M", "-PT24H", "PT36H", "P1MT1H", "P1Y2M3W4DT5H6M7S", "PT0S",
    "P10000Y", "PT86399.999999999S", "-PT86400.000000001S", "P2W", "P12M", "P100Y",
  ];
  const addDates = ["2021-01-31", "2020-02-29", "2020-12-31", "9999-12-01", "-009999-01-31"];
  for (const op of ["add", "subtract"]) {
    for (const d of addDates) {
      for (const dur of durations) {
        out.push({ op, receiver: d, args: [dur] });
      }
    }
    for (const d of ["2021-01-31", "2020-02-29", "2020-03-31"]) {
      for (const dur of ["P1M", "-P1M", "P1Y", "P1Y1M"]) {
        out.push({ op, receiver: d, args: [dur], options: { overflow: "reject" } });
      }
    }
  }

  // Differences: every unit pair x rounding mode, with sampled dates and increments.
  const pool = ["2020-01-01", "2020-02-29", "2021-01-31", "2019-03-15", "2006-08-24",
    "2019-01-31", "2024-12-31", "2016-02-29", "2010-07-04", "1999-12-31", "2021-03-01"];
  for (const op of ["until", "since"]) {
    for (const [largestUnit, smallestUnit] of unitPairs(DATE_UNITS)) {
      for (const roundingMode of ROUNDING_MODES) {
        const roundingIncrement = r.next() < 0.5 ? 1 : r.pick([2, 3, 5, 10]);
        out.push({
          op, receiver: r.pick(pool), args: [r.pick(pool)],
          options: opts({ largestUnit, smallestUnit, roundingMode, roundingIncrement }),
        });
      }
    }
    for (const [a, b] of [["2006-08-24", "2019-01-31"], ["2019-01-31", "2006-08-24"],
      ["2020-02-29", "2021-02-28"], ["2020-01-31", "2020-03-01"], ["2020-03-31", "2020-02-29"]]) {
      out.push({ op, receiver: a, args: [b] });
      for (const largestUnit of DATE_UNITS) out.push({ op, receiver: a, args: [b], options: { largestUnit } });
      out.push({ op, receiver: a, args: [b], options: { largestUnit: "hour" } });
      out.push({ op, receiver: a, args: [b], options: { smallestUnit: "hour" } });
    }
  }

  // with().
  const fields = [{ day: 31 }, { month: 2 }, { month: 2, day: 31 }, { year: 2019 },
    { year: 2021, month: 13 }, { day: 0 }, { month: 4, day: 31 }, { year: 10000 }];
  for (const d of ["2020-01-31", "2020-02-29", "2021-06-15"]) {
    for (const f of fields) {
      out.push({ op: "with", receiver: d, args: [f] });
      out.push({ op: "with", receiver: d, args: [f], options: { overflow: "reject" } });
    }
  }

  // toString().
  for (const d of ["2020-01-01", "-000001-12-31", "9999-12-31", "0000-06-30"]) {
    for (const calendarName of [undefined, "auto", "always", "never", "critical"]) {
      out.push({ op: "toString", receiver: d, options: opts({ calendarName }) });
    }
  }

  // compare / equals.
  for (let k = 0; k < 20; k++) {
    const a = r.pick(DATES);
    const b = k % 4 === 0 ? a : r.pick(DATES);
    out.push({ op: "compare", receiver: a, args: [b] });
    out.push({ op: "equals", receiver: a, args: [b] });
  }

  // Accessors.
  const getters = ["year", "month", "day", "dayOfWeek", "dayOfYear", "weekOfYear", "yearOfWeek",
    "daysInWeek", "daysInMonth", "daysInYear", "monthsInYear", "inLeapYear"];
  for (const d of ["2020-01-01", "2021-01-03", "2020-12-31", "2019-12-30", "2024-02-29",
    "2027-01-01", "-000001-01-01", "1900-12-31"]) {
    for (const g of getters) out.push({ op: "get", receiver: d, args: [g] });
  }

  // Conversion to a zoned date-time (start of day).
  for (const [d, tz] of [["2020-01-01", "UTC"], ["2015-03-29", "America/Sao_Paulo"],
    ["2018-11-04", "America/Sao_Paulo"], ["2011-12-30", "Pacific/Apia"],
    ["2020-03-08", "America/New_York"], ["2020-06-01", "+05:30"]]) {
    out.push({ op: "toZonedDateTime", receiver: d, args: [tz] });
  }

  return out;
}
