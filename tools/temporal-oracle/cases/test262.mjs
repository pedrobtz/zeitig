// A curated slice of test262 (tc39/test262 @ 7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd,
// test/built-ins/Temporal/) transcribed into case records. Only the INPUTS are
// transcribed: generate.mjs computes the expected values with the reference
// polyfill, which passes test262. Loops in the original files are expanded
// here (deterministically). Temporal objects built from epoch nanoseconds or
// constructor arguments are written as their toString() form.

const SRC = "test262/test/built-ins/Temporal/";
const ALL_MODES = ["ceil", "floor", "expand", "trunc", "halfCeil", "halfFloor", "halfExpand",
  "halfTrunc", "halfEven"];
const DATE_UNITS = ["year", "month", "week", "day"];
const TIME_UNITS = ["hour", "minute", "second", "millisecond", "microsecond", "nanosecond"];

export default function cases() {
  const out = [];
  const push = (type, path, op, receiver, args, options) => {
    const c = { type, op, receiver };
    if (args !== undefined) c.args = args;
    if (options !== undefined) c.options = options;
    c.source = SRC + path;
    out.push(c);
  };

  // ---------------------------------------------------------------------------
  // PlainDate
  // ---------------------------------------------------------------------------

  // PlainDate#until rounding to each date unit with roundingMode halfEven, both directions.
  {
    const path = "PlainDate/prototype/until/roundingmode-halfEven.js";
    for (const smallestUnit of DATE_UNITS) {
      const options = { smallestUnit, roundingMode: "halfEven" };
      push("plain-date", path, "until", "2019-01-08", ["2021-09-07"], options);
      push("plain-date", path, "until", "2021-09-07", ["2019-01-08"], options);
    }
  }

  // PlainDate#until exactly on a .5 boundary of years / months, every rounding mode.
  {
    const path = "PlainDate/prototype/until/roundingmode-half-boundary.js";
    const pairs = [
      ["2019-01-01", "2020-07-02", "year"], ["2018-01-01", "2020-07-02", "year"],
      ["2019-01-01", "2019-02-15", "month"], ["2018-12-01", "2019-02-15", "month"],
    ];
    for (const [a, b, smallestUnit] of pairs) {
      push("plain-date", path, "until", a, [b]);
      for (const roundingMode of ALL_MODES) {
        push("plain-date", path, "until", a, [b], { smallestUnit, roundingMode });
      }
    }
  }

  // PlainDate#until where rounding up 11 months balances into a year.
  push("plain-date", "PlainDate/prototype/until/round-cross-unit-boundary.js", "until",
    "2022-01-01", ["2023-12-25"],
    { largestUnit: "year", smallestUnit: "month", roundingMode: "expand" });

  // PlainDate#until / #since month rounding relative to the receiver; month-end clamping.
  {
    const relCases = [
      ["2019-03-01", "2019-01-29"], ["2019-01-29", "2019-03-01"],
      ["2019-03-29", "2019-01-30"], ["2019-01-30", "2019-03-29"],
      ["2019-03-30", "2019-01-31"], ["2019-01-31", "2019-03-30"],
      ["2019-03-31", "2019-01-31"], ["2019-01-31", "2019-03-31"],
    ];
    const hx = { smallestUnit: "month", roundingMode: "halfExpand" };
    let path = "PlainDate/prototype/until/rounding-relative.js";
    push("plain-date", path, "until", "2019-01-01", ["2019-02-15"], hx);
    push("plain-date", path, "until", "2019-02-15", ["2019-01-01"], hx);
    for (const [end, start] of relCases) {
      push("plain-date", path, "until", start, [end], { largestUnit: "month" });
    }
    path = "PlainDate/prototype/since/rounding-relative.js";
    push("plain-date", path, "since", "2019-02-15", ["2019-01-01"], hx);
    push("plain-date", path, "since", "2019-01-01", ["2019-02-15"], hx);
    for (const [end, start] of relCases) {
      push("plain-date", path, "since", end, [start], { largestUnit: "month" });
    }
  }

  // PlainDate#until across leap days with different largestUnits.
  {
    const path = "PlainDate/prototype/until/leap-year-arithmetic.js";
    const tests = [
      ["2021-01-07", "2021-03-07", ["year", "month"]],
      ["1960-02-16", "2021-03-15", ["year"]],
      ["1960-02-16", "2020-03-15", ["year"]],
      ["2021-03-15", "1960-02-16", ["year"]],
      ["2020-03-15", "1960-02-16", ["year"]],
      ["2021-03-07", "2021-01-07", ["year", "month"]],
      ["2020-02-01", "2021-02-01", ["week"]],
      ["2021-02-28", "2022-02-28", ["week"]],
      ["2019-01-01", "2020-01-01", ["day"]],
      ["2020-01-01", "2021-01-01", ["day"]],
      ["2019-06-01", "2020-06-01", ["day"]],
      ["2020-06-01", "2021-06-01", ["day"]],
      ["2019-02-01", "2019-03-01", ["day"]],
      ["2020-02-01", "2020-03-01", ["day"]],
    ];
    for (const [one, two, units] of tests) {
      for (const largestUnit of units) {
        push("plain-date", path, "until", one, [two], { largestUnit });
      }
    }
  }

  // PlainDate#add with years / months / weeks / days, including end-of-month clamping.
  {
    const path = "PlainDate/prototype/add/basic.js";
    for (const [d, dur] of [
      ["1976-11-18", "P43Y"], ["1976-11-18", "P3M"], ["1976-11-18", "P20D"],
      ["2019-01-31", "P1M"], ["2019-11-18", "-P43Y"], ["1977-02-18", "-P3M"],
      ["1976-12-08", "-P20D"], ["2019-02-28", "-P1M"],
    ]) {
      push("plain-date", path, "add", d, [dur]);
    }
    const testData = [
      ["2020-02-29", "P1Y"], ["2020-02-29", "P4Y"], ["2021-07-16", "P1Y"],
      ["2021-07-16", "P5M"], ["2021-08-16", "P5M"], ["2021-10-31", "P5M"],
      ["2021-09-30", "P5M"], ["2019-09-30", "P5M"], ["2019-10-01", "P5M"],
      ["2021-07-16", "P1Y2M"], ["2021-11-30", "P1Y2M"], ["2021-12-31", "P1Y2M"],
      ["2022-12-31", "P1Y2M"], ["2021-07-16", "P1Y4D"], ["2021-02-27", "P1Y4D"],
      ["2023-02-27", "P1Y4D"], ["2021-12-30", "P1Y4D"], ["2021-07-30", "P1Y4D"],
      ["2021-06-30", "P1Y4D"], ["2021-07-16", "P1Y2M4D"], ["2021-02-27", "P1Y2M4D"],
      ["2021-02-26", "P1Y2M4D"], ["2023-02-26", "P1Y2M4D"], ["2021-12-30", "P1Y2M4D"],
      ["2021-07-30", "P1Y2M4D"], ["2021-06-30", "P1Y2M4D"], ["2021-07-16", "P10D"],
      ["2021-07-26", "P10D"], ["2021-12-26", "P10D"], ["2020-02-26", "P10D"],
      ["2021-02-26", "P10D"], ["2020-02-19", "P10D"], ["2021-02-19", "P10D"],
      ["2021-02-19", "P1W"], ["2021-02-27", "P1W"], ["2020-02-27", "P1W"],
      ["2021-12-24", "P1W"], ["2021-12-27", "P1W"], ["2021-01-27", "P1W"],
      ["2021-06-27", "P1W"], ["2021-07-27", "P1W"], ["2021-02-19", "P6W"],
      ["2021-02-27", "P6W"], ["2020-02-27", "P6W"], ["2021-12-24", "P6W"],
      ["2021-12-27", "P6W"], ["2021-01-27", "P6W"], ["2021-06-27", "P6W"],
      ["2021-07-27", "P6W"], ["2020-02-29", "P2W3D"], ["2020-02-28", "P2W3D"],
      ["2021-02-28", "P2W3D"], ["2020-12-28", "P2W3D"], ["2020-02-29", "P1Y2W"],
      ["2020-02-28", "P1Y2W"], ["2021-02-28", "P1Y2W"], ["2020-12-28", "P1Y2W"],
      ["2020-02-29", "P2M3W"], ["2020-02-28", "P2M3W"], ["2021-02-28", "P2M3W"],
      ["2020-12-28", "P2M3W"], ["2019-12-28", "P2M3W"], ["2019-10-28", "P2M3W"],
      ["2019-10-31", "P2M3W"],
    ];
    for (const [d, dur] of testData) push("plain-date", path, "add", d, [dur]);
  }

  // ---------------------------------------------------------------------------
  // PlainDateTime
  // ---------------------------------------------------------------------------

  // PlainDateTime#round to each unit from day down with roundingMode halfExpand.
  {
    const path = "PlainDateTime/prototype/round/roundingmode-halfExpand.js";
    for (const smallestUnit of ["day", ...TIME_UNITS]) {
      push("plain-date-time", path, "round", "1976-11-18T14:23:30.1239875",
        undefined, { smallestUnit, roundingMode: "halfExpand" });
    }
  }

  // PlainDateTime#round in a negative year rounds towards the Big Bang, not the epoch.
  {
    const path = "PlainDateTime/prototype/round/rounding-direction.js";
    for (const roundingMode of ["floor", "trunc", "ceil", "halfExpand"]) {
      push("plain-date-time", path, "round", "-000099-12-15T12:00:00.5",
        undefined, { smallestUnit: "second", roundingMode });
    }
  }

  // PlainDateTime#round of 23:59:59.999999999 balances into the next day.
  {
    const path = "PlainDateTime/prototype/round/balance.js";
    for (const smallestUnit of ["day", "hour", "minute", "second", "millisecond", "microsecond"]) {
      push("plain-date-time", path, "round", "1976-11-18T23:59:59.999999999",
        undefined, { smallestUnit });
    }
  }

  // PlainDateTime#round past the representable limits throws RangeError.
  {
    const path = "PlainDateTime/prototype/round/limits.js";
    for (const smallestUnit of ["day", "hour", "minute", "second", "millisecond", "microsecond"]) {
      push("plain-date-time", path, "round", "-271821-04-19T00:00:00.000000001",
        undefined, { smallestUnit, roundingMode: "floor" });
      push("plain-date-time", path, "round", "+275760-09-13T23:59:59.999999999",
        undefined, { smallestUnit, roundingMode: "ceil" });
    }
  }

  // PlainDateTime#round to the hour before 1970, every rounding mode.
  {
    const path = "PlainDateTime/prototype/round/negative-time.js";
    for (const roundingMode of ALL_MODES) {
      push("plain-date-time", path, "round", "1938-04-24T22:13:20",
        undefined, { smallestUnit: "hour", roundingIncrement: 1, roundingMode });
    }
  }

  // PlainDateTime#until where rounding up balances into the next larger unit.
  {
    const path = "PlainDateTime/prototype/until/round-cross-unit-boundary.js";
    push("plain-date-time", path, "until", "2022-01-01T00:00", ["2023-12-25T00:00"],
      { largestUnit: "year", smallestUnit: "month", roundingMode: "expand" });
    push("plain-date-time", path, "until", "2000-05-02T00:00", ["2000-05-02T01:59:59"],
      { largestUnit: "hour", smallestUnit: "minute", roundingMode: "expand" });
    push("plain-date-time", path, "until", "1970-01-01T00:00",
      ["1971-12-31T23:59:59.999999999"],
      { largestUnit: "year", smallestUnit: "microsecond", roundingMode: "expand" });
  }

  // ---------------------------------------------------------------------------
  // PlainTime
  // ---------------------------------------------------------------------------

  // PlainTime#round to each unit with roundingMode halfEven.
  {
    const path = "PlainTime/prototype/round/roundingmode-halfEven.js";
    for (const smallestUnit of TIME_UNITS) {
      push("plain-time", path, "round", "13:46:23.1239875",
        undefined, { smallestUnit, roundingMode: "halfEven" });
    }
  }

  // PlainTime#round of 23:59:59.999999999 wraps to midnight.
  {
    const path = "PlainTime/prototype/round/rounding-cross-midnight.js";
    for (const smallestUnit of ["hour", "minute", "second", "millisecond", "microsecond"]) {
      push("plain-time", path, "round", "23:59:59.999999999", undefined, { smallestUnit });
    }
  }

  // PlainTime#round to hour increments dividing 24.
  {
    const path = "PlainTime/prototype/round/roundingincrement-hours.js";
    for (const roundingIncrement of [1, 2, 3, 4, 6, 8, 12]) {
      push("plain-time", path, "round", "03:34:56.987654321",
        undefined, { smallestUnit: "hour", roundingIncrement });
    }
  }

  // ---------------------------------------------------------------------------
  // Instant
  // ---------------------------------------------------------------------------

  // Instant#round to each time unit with roundingMode halfEven.
  {
    const path = "Instant/prototype/round/roundingmode-halfEven.js";
    for (const smallestUnit of TIME_UNITS) {
      push("instant", path, "round", "1976-11-18T14:23:30.1239875Z",
        undefined, { smallestUnit, roundingMode: "halfEven" });
    }
  }

  // Instant#round in a negative year rounds towards the Big Bang, not the epoch.
  {
    const path = "Instant/prototype/round/rounding-direction.js";
    for (const roundingMode of ["floor", "trunc", "ceil", "halfExpand"]) {
      push("instant", path, "round", "-000099-12-15T12:00:00.5Z",
        undefined, { smallestUnit: "second", roundingMode });
    }
  }

  // Instant#round with increments equal to one solar day.
  {
    const path = "Instant/prototype/round/round-to-days.js";
    for (const [smallestUnit, roundingIncrement] of [
      ["hour", 24], ["minute", 1440], ["second", 86400], ["millisecond", 86400000],
    ]) {
      push("instant", path, "round", "1976-11-18T14:23:30.123456789Z",
        undefined, { smallestUnit, roundingIncrement });
    }
  }

  // Instant#round to the hour before the epoch, every rounding mode.
  {
    const path = "Instant/prototype/round/negative-instant.js";
    for (const roundingMode of ALL_MODES) {
      push("instant", path, "round", "1938-04-24T22:13:20Z",
        undefined, { smallestUnit: "hour", roundingIncrement: 1, roundingMode });
    }
  }

  // ---------------------------------------------------------------------------
  // ZonedDateTime
  // ---------------------------------------------------------------------------

  // ZonedDateTime#until rounding to every unit with roundingMode halfExpand, both directions.
  {
    const path = "ZonedDateTime/prototype/until/roundingmode-halfExpand.js";
    const earlier = "2019-01-08T08:22:36.123456789+00:00[UTC]";
    const later = "2021-09-07T12:39:40.987654289+00:00[UTC]";
    for (const smallestUnit of [...DATE_UNITS, ...TIME_UNITS]) {
      const options = { smallestUnit, roundingMode: "halfExpand" };
      push("zoned-date-time", path, "until", earlier, [later], options);
      push("zoned-date-time", path, "until", later, [earlier], options);
    }
  }

  // ZonedDateTime#until rounding to increments of time units.
  {
    const path = "ZonedDateTime/prototype/until/rounding-increments.js";
    for (const [smallestUnit, roundingIncrement] of [
      ["hour", 3], ["minute", 30], ["second", 15], ["millisecond", 10],
      ["microsecond", 10], ["nanosecond", 10],
    ]) {
      push("zoned-date-time", path, "until", "2019-01-08T09:22:36.123456789+01:00[+01:00]",
        ["2021-09-07T13:39:40.987654321+01:00[+01:00]"],
        { smallestUnit, roundingIncrement, roundingMode: "halfExpand" });
    }
  }

  // ZonedDateTime#until where rounding up balances into the next larger unit.
  {
    const path = "ZonedDateTime/prototype/until/round-cross-unit-boundary.js";
    push("zoned-date-time", path, "until", "2022-01-01T00:00:00+00:00[UTC]",
      ["2023-12-25T00:00:00+00:00[UTC]"],
      { largestUnit: "year", smallestUnit: "month", roundingMode: "expand" });
    push("zoned-date-time", path, "until", "1970-01-01T00:00:00+00:00[UTC]",
      ["1970-01-01T01:59:59+00:00[UTC]"],
      { largestUnit: "hour", smallestUnit: "minute", roundingMode: "expand" });
    push("zoned-date-time", path, "until", "1970-01-01T00:00:00+00:00[UTC]",
      ["1971-12-31T23:59:59.999999999+00:00[UTC]"],
      { largestUnit: "year", smallestUnit: "microsecond", roundingMode: "expand" });
  }

  // ZonedDateTime#until and #since agree.
  {
    const path = "ZonedDateTime/prototype/until/until-since.js";
    const a = "1976-11-18T15:23:30.123456789+01:00[+01:00]";
    const b = "2016-03-03T18:00:00+01:00[+01:00]";
    push("zoned-date-time", path, "until", a, [b]);
    push("zoned-date-time", path, "since", b, [a]);
  }

  // ZonedDateTime#round to each unit with roundingMode halfExpand.
  {
    const path = "ZonedDateTime/prototype/round/roundingmode-halfExpand.js";
    for (const smallestUnit of ["day", "minute", "second", "millisecond", "microsecond",
      "nanosecond"]) {
      push("zoned-date-time", path, "round", "1976-11-18T15:23:30.1239875+01:00[+01:00]",
        undefined, { smallestUnit, roundingMode: "halfExpand" });
    }
  }

  // ZonedDateTime#round of 23:59:59.999999999 balances into the next day.
  {
    const path = "ZonedDateTime/prototype/round/smallestunit.js";
    for (const smallestUnit of ["day", "hour", "minute", "second", "millisecond",
      "microsecond"]) {
      push("zoned-date-time", path, "round", "1976-11-18T23:59:59.999999999+01:00[+01:00]",
        undefined, { smallestUnit });
    }
  }

  // ZonedDateTime#add of years / months / weeks / days in UTC (wall clock 12:34).
  {
    const path = "ZonedDateTime/prototype/add/basic-arithmetic.js";
    const z = (d) => `${d}T12:34:00+00:00[UTC]`;
    const tests = [
      // Years
      ["2021-07-16", "P1Y"], ["2021-07-16", "P4Y"], ["2021-07-16", "-P1Y"],
      ["2021-07-16", "-P4Y"], ["1997-12-01", "P3Y6M17D"], ["2001-06-18", "-P3Y6M17D"],
      // Months
      ["2021-07-16", "P5M"], ["2021-08-16", "P5M"], ["2019-10-01", "P5M"],
      ["2021-10-31", "P5M"], ["2021-07-16", "P1Y2M"], ["2021-11-30", "P1Y2M"],
      ["2021-07-16", "-P5M"], ["2021-01-16", "-P5M"], ["2019-02-01", "-P5M"],
      ["2021-03-31", "-P5M"], ["2021-07-16", "-P1Y2M"], ["2021-02-17", "-P1Y2M"],
      ["2000-12-01", "P6M"], ["2001-06-01", "-P6M"],
      // Weeks
      ["2021-02-19", "P1W"], ["2021-12-24", "P1W"], ["2021-12-25", "P1W"],
      ["2021-01-27", "P1W"], ["2021-07-27", "P1W"], ["2021-06-27", "P1W"],
      ["2021-01-27", "P6W"], ["2021-12-24", "P6W"], ["2021-06-27", "P6W"],
      ["2021-07-27", "P6W"], ["2020-12-28", "P1Y2W"], ["2019-10-28", "P2M3W"],
      ["2019-10-31", "P2M3W"], ["2021-02-19", "-P1W"], ["2021-01-08", "-P1W"],
      ["2021-01-07", "-P1W"], ["2021-06-04", "-P1W"], ["2021-07-03", "-P1W"],
      ["2021-06-04", "-P6W"], ["2021-01-27", "-P6W"], ["2021-09-08", "-P6W"],
      ["2021-08-08", "-P6W"], ["2022-01-05", "-P1Y2W"], ["2019-03-02", "-P2M3W"],
      ["2000-01-01", "P40W"], ["2000-10-07", "-P40W"],
      // Days
      ["2021-07-16", "P10D"], ["2021-07-26", "P10D"], ["2021-12-26", "P10D"],
      ["2020-12-28", "P2W3D"], ["2021-07-16", "P1Y2M4D"], ["2021-02-27", "P1Y2M4D"],
      ["2021-07-30", "P1Y2M4D"], ["2021-01-28", "P1Y2M4D"], ["2021-06-30", "P1Y2M4D"],
      ["2021-07-16", "-P10D"], ["2021-07-06", "-P10D"], ["2021-01-04", "-P10D"],
      ["2021-01-15", "-P2W3D"], ["2021-07-16", "-P1Y2M4D"], ["2021-07-04", "-P1Y2M4D"],
      ["2021-06-04", "-P1Y2M4D"], ["2000-01-01", "P280D"], ["2000-10-07", "-P280D"],
    ];
    for (const [d, dur] of tests) push("zoned-date-time", path, "add", z(d), [dur]);
  }

  // ZonedDateTime#add / #subtract of hours + nanoseconds across the Unix epoch.
  {
    const path = "ZonedDateTime/prototype/add/cross-epoch.js";
    const zdt = "1969-12-25T12:23:45.678901234+00:00[UTC]";
    const one = "1969-12-15T12:23:45.678900434+00:00[UTC]";
    const two = "1970-01-04T12:23:45.678902034+00:00[UTC]";
    push("zoned-date-time", path, "subtract", zdt, ["PT240H0.0000008S"]);
    push("zoned-date-time", path, "add", zdt, ["PT240H0.0000008S"]);
    push("zoned-date-time", path, "subtract", two, ["PT480H0.0000016S"]);
    push("zoned-date-time", path, "add", one, ["PT480H0.0000016S"]);
  }

  // ZonedDateTime#add of one month from Jan 31 constrains, or throws with overflow reject.
  {
    const jan31 = "2020-01-31T15:00:00-08:00[-08:00]";
    let path = "ZonedDateTime/prototype/add/constrain-when-ambiguous-result.js";
    push("zoned-date-time", path, "add", jan31, ["P1M"]);
    push("zoned-date-time", path, "add", jan31, ["P1M"], { overflow: "constrain" });
    path = "ZonedDateTime/prototype/add/throw-when-ambiguous-result-with-reject.js";
    push("zoned-date-time", path, "add", jan31, ["P1M"], { overflow: "reject" });
  }

  // ---------------------------------------------------------------------------
  // Duration
  // ---------------------------------------------------------------------------

  // Duration#round to every unit with roundingMode halfExpand, relative to a PlainDate.
  {
    const path = "Duration/prototype/round/roundingmode-halfExpand.js";
    for (const smallestUnit of [...DATE_UNITS, ...TIME_UNITS]) {
      push("duration", path, "round", "P5Y6M7W8DT40H30M20.1239875S", undefined,
        { smallestUnit, relativeTo: "2020-04-01", roundingMode: "halfExpand" });
      push("duration", path, "round", "-P5Y6M7W8DT40H30M20.1239875S", undefined,
        { smallestUnit, relativeTo: "2020-12-01", roundingMode: "halfExpand" });
    }
  }

  // Duration#round where rounding up balances into the next larger unit.
  {
    const path = "Duration/prototype/round/round-cross-unit-boundary.js";
    const o = { smallestUnit: "month", roundingMode: "expand", relativeTo: "2022-01-01" };
    push("duration", path, "round", "P1Y11M24D", undefined, o);
    push("duration", path, "round", "-P1Y11M24D", undefined, o);
    push("duration", path, "round", "PT1H59M59.9S", undefined,
      { smallestUnit: "second", roundingMode: "expand" });
    push("duration", path, "round", "-PT1H59M59.9S", undefined,
      { smallestUnit: "second", roundingMode: "expand" });
    push("duration", path, "round", "P11M24D", undefined, o);
  }

  // Duration#round of days to months / years depends on the relativeTo date.
  {
    const path = "Duration/prototype/round/relativeto-rounding-date.js";
    push("duration", path, "round", "P45D", undefined,
      { relativeTo: "2019-01-01", smallestUnit: "month" });
    push("duration", path, "round", "-P45D", undefined,
      { relativeTo: "2019-02-15", smallestUnit: "month" });
    for (const relativeTo of ["2018-01-01", "2018-07-01", "2019-01-01", "2020-01-01",
      "2020-07-01"]) {
      push("duration", path, "round", "P547DT12H", undefined,
        { relativeTo, smallestUnit: "year" });
    }
  }

  // Duration#round balancing 40 days up to months relative to different dates.
  {
    const path = "Duration/prototype/round/relativeto-balances-up-differently-depending-on-relative-date.js";
    for (const receiver of ["P40D", "-P40D"]) {
      for (const relativeTo of ["2020-01-01", "2020-02-01", "2020-03-01", "2020-04-01"]) {
        push("duration", path, "round", receiver, undefined,
          { largestUnit: "year", relativeTo });
      }
    }
  }

  // Duration#round balancing days into both years and months.
  push("duration", "Duration/prototype/round/balances-days-up-to-both-years-and-months.js",
    "round", "P11M396D", undefined, { largestUnit: "year", relativeTo: "2017-01-01" });

  // Duration#round of 11 months from May 31 with ceil does not overshoot.
  push("duration", "Duration/prototype/round/end-of-month-round-up.js", "round", "P11M",
    undefined, { relativeTo: "2023-05-31", smallestUnit: "month", roundingMode: "ceil" });

  // Duration#round balancing to years starting just after a leap February.
  {
    const path = "Duration/prototype/round/february-leap-year.js";
    for (const receiver of ["P3Y11M27D", "P3Y11M28D", "P3Y11M29D"]) {
      push("duration", path, "round", receiver, undefined,
        { largestUnit: "year", relativeTo: "1972-03-01" });
    }
  }

  // Duration#round rebalancing 100 days expressed in each unit up to months. (The
  // milliseconds / microseconds / nanoseconds variants have no distinct ISO 8601 string
  // form: they parse to the same seconds field as PT8640000S.)
  {
    const path = "Duration/prototype/round/largestunit-correct-rebalancing.js";
    for (const receiver of ["P100D", "PT2400H", "PT144000M", "PT8640000S"]) {
      push("duration", path, "round", receiver, undefined,
        { relativeTo: "2023-02-21", largestUnit: "month" });
    }
  }

  // Duration#round without relativeTo: 60 negative hours to days; 25 hours balance to days.
  push("duration", "Duration/prototype/round/round-negative-result.js", "round", "-PT60H",
    undefined, { smallestUnit: "day" });
  push("duration", "Duration/prototype/round/days-24-hours.js", "round", "PT25H", undefined,
    { largestUnit: "day" });
  push("duration", "Duration/prototype/round/relativeto-days-24-hours-relative-to-zoned-date-time.js",
    "round", "PT25H", undefined,
    { largestUnit: "day", relativeTo: "2001-09-09T06:16:40+04:30[+04:30]" });

  // Duration#total of a days-and-time duration in each unit, no relativeTo.
  {
    const path = "Duration/prototype/total/total-of-each-unit.js";
    for (const unit of ["day", ...TIME_UNITS]) {
      push("duration", path, "total", "P5DT5H5M5.005005005S", undefined, { unit });
    }
  }

  // Duration#total of a full duration in each unit relative to a PlainDate and a ZonedDateTime.
  {
    const path = "Duration/prototype/total/relativeto-total-of-each-unit.js";
    for (const unit of [...DATE_UNITS, ...TIME_UNITS]) {
      for (const relativeTo of ["2000-01-01", "1972-01-01T00:00:00+00:00[UTC]"]) {
        push("duration", path, "total", "P5Y5M5W5DT5H5M5.005005005S", undefined,
          { unit, relativeTo });
      }
    }
  }

  // Duration#total in months of +/-40 days depends on the relativeTo date.
  {
    const path = "Duration/prototype/total/relativeto-calendar-units-depend-on-relative-date.js";
    push("duration", path, "total", "P40D", undefined, { unit: "month", relativeTo: "2020-02-01" });
    push("duration", path, "total", "P40D", undefined, { unit: "month", relativeTo: "2020-01-01" });
    push("duration", path, "total", "-P40D", undefined, { unit: "month", relativeTo: "2020-03-01" });
    push("duration", path, "total", "-P40D", undefined, { unit: "month", relativeTo: "2020-04-01" });
  }

  // Duration#total in years balancing days into both years and months.
  {
    const path = "Duration/prototype/total/balances-days-up-to-both-years-and-months.js";
    push("duration", path, "total", "P11M396D", undefined, { unit: "year", relativeTo: "2017-01-01" });
    push("duration", path, "total", "-P11M396D", undefined, { unit: "year", relativeTo: "2017-01-01" });
  }

  // Duration#total of a duration with calendar units throws without relativeTo.
  {
    const path = "Duration/prototype/total/rounds-durations-with-calendar-units.js";
    for (const unit of [...DATE_UNITS, ...TIME_UNITS]) {
      push("duration", path, "total", "P5Y5M5W5DT5H5M5.005005005S", undefined, { unit });
    }
  }

  // Duration#total in days of 1 week 1 hour relative to a ZonedDateTime.
  push("duration", "Duration/prototype/total/relativeto-zoneddatetime-with-fractional-days.js",
    "total", "P1WT1H", undefined, { relativeTo: "1970-01-01T00:00:00+00:00[UTC]", unit: "day" });

  return out;
}
