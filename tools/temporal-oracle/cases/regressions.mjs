// Cases for each difference the conformance harness found and zeitig fixed,
// so that sampling changes elsewhere cannot drop them. Exact `halfEven` ties
// and similar edge inputs are rare in the sampled cases.

const MODES = ["halfEven", "halfExpand", "halfTrunc", "ceil", "floor"];

export default function cases() {
  const out = [];
  const add = (type, op, receiver, args, options) => out.push({ type, op, receiver, args, options });

  // [!u-ca=iso8601] is accepted; other calendars are an error in zeitig.
  for (const [type, s] of [["plain-date", "2020-01-01[!u-ca=iso8601]"],
    ["plain-date-time", "2020-01-01T12:00[!u-ca=iso8601]"],
    ["zoned-date-time", "2020-01-01T12:00+01:00[Europe/Paris][!u-ca=iso8601]"],
    ["zoned-date-time", "2020-01-01T12:00+01:00[Europe/Paris][u-ca=japanese]"],
    ["plain-time", "12:00[!u-ca=iso8601]"], ["instant", "2020-01-01T12:00Z[!u-ca=iso8601]"]]) {
    add(type, "from", s);
  }
  // UTC offsets of 24 hours or more, and fractional-second offsets.
  for (const s of ["2020-01-01T12:00+24:00", "2020-01-01T12:00-24:00", "2020-01-01T12:00+23:59:59.999999999"]) {
    add("plain-date-time", "from", s);
    add("instant", "from", s);
  }
  add("zoned-date-time", "from", "2020-01-01T12:00+24:00[UTC]");
  add("zoned-date-time", "from", "2020-01-01T12:00+00:00:00.5[UTC]");
  // `+00:00` is a time zone of its own, not UTC.
  add("zoned-date-time", "from", "2020-01-01T12:00+00:00[+00:00]");
  add("zoned-date-time", "withTimeZone", "2020-01-01T12:00+00:00[UTC]", ["+00:00"]);
  add("zoned-date-time", "get", "2020-01-01T12:00+00:00[+00:00]", ["timeZoneId"]);
  add("zoned-date-time", "until", "2020-01-01T12:00+00:00[UTC]", ["2020-01-03T12:00+00:00[+00:00]"], { largestUnit: "day" });
  add("zoned-date-time", "equals", "2020-01-01T12:00+00:00[UTC]", ["2020-01-01T12:00+00:00[Etc/GMT]"]);
  // Out-of-range epoch nanoseconds (used to panic).
  for (const ns of ["-8640000000000000000001", "100000000000000000000000000"]) {
    add("instant", "fromEpochNanoseconds", undefined, [ns]);
  }
  // Instants round "as if positive": trunc goes down before 1970.
  for (const roundingMode of ["trunc", "expand", "halfTrunc", "halfExpand", "halfEven"]) {
    add("instant", "round", "1969-12-31T23:59:59.5Z", undefined, { smallestUnit: "second", roundingMode });
    add("instant", "round", "1969-12-31T23:30Z", undefined, { smallestUnit: "hour", roundingMode });
    add("instant", "toString", "1969-12-31T23:59:59.999Z", undefined, { fractionalSecondDigits: 1, roundingMode });
  }
  // since() is the negated until() with the negated rounding mode.
  for (const roundingMode of MODES) {
    for (const op of ["until", "since"]) {
      add("plain-date", op, "2019-03-15", ["2021-03-01"], { largestUnit: "month", smallestUnit: "month", roundingMode });
      add("plain-date", op, "2021-03-01", ["2019-03-15"], { largestUnit: "year", smallestUnit: "month", roundingMode });
      add("plain-date-time", op, "2021-01-31T12:00", ["2020-01-01T00:00"], { largestUnit: "month", smallestUnit: "day", roundingMode });
      add("zoned-date-time", op, "2020-03-31T00:00+01:00[Europe/London]", ["2020-01-31T00:00+00:00[Europe/London]"],
        { largestUnit: "month", smallestUnit: "day", roundingMode });
    }
  }
  // halfEven ties on calendar units and days.
  add("plain-date", "until", "2019-01-01", ["2019-01-16"], { largestUnit: "month", smallestUnit: "month", roundingMode: "halfEven" });
  add("plain-date-time", "until", "2020-01-01T00:00", ["2020-01-02T12:00"], { smallestUnit: "day", roundingMode: "halfEven" });
  add("plain-date-time", "until", "2020-01-01T00:00", ["2020-01-03T12:00"], { smallestUnit: "day", roundingMode: "halfEven" });
  add("zoned-date-time", "until", "2021-03-13T12:00-08:00[America/Los_Angeles]", ["2021-03-15T00:00-07:00[America/Los_Angeles]"],
    { largestUnit: "day", smallestUnit: "day", roundingMode: "halfEven" });
  for (const relativeTo of ["2021-03-14T01:00-08:00[America/Los_Angeles]", "2021-03-14T01:00", "2020-01-01"]) {
    add("duration", "round", "P1M1DT12H", undefined, { largestUnit: "month", smallestUnit: "day", roundingMode: "halfEven", relativeTo });
    add("duration", "round", "-PT36H", undefined, { smallestUnit: "day", roundingMode: "halfEven", relativeTo });
    add("duration", "round", "P1Y6M", undefined, { smallestUnit: "year", roundingMode: "halfEven", relativeTo });
  }
  // halfEven ties when rounding wall-clock times: the parity is that of the
  // rounded field.
  for (const [type, s] of [["plain-time", "12:00:00.0000035"], ["plain-date-time", "1995-12-07T03:24:30.0000035"],
    ["zoned-date-time", "1995-12-07T03:24:30.0000035+00:00[UTC]"]]) {
    add(type, "round", s, undefined, { smallestUnit: "nanosecond", roundingIncrement: 40, roundingMode: "halfEven" });
    add(type, "round", s, undefined, { smallestUnit: "microsecond", roundingIncrement: 2, roundingMode: "halfEven" });
  }
  for (const [type, s] of [["plain-time", "01:30"], ["plain-date-time", "2020-01-01T01:30"],
    ["zoned-date-time", "2020-01-01T01:30+00:00[UTC]"]]) {
    add(type, "round", s, undefined, { smallestUnit: "minute", roundingIncrement: 20, roundingMode: "halfEven" });
    add(type, "round", s, undefined, { smallestUnit: "hour", roundingIncrement: 3, roundingMode: "halfEven" });
  }
  add("plain-date-time", "round", "2020-01-01T12:00", undefined, { smallestUnit: "day", roundingMode: "halfEven" });
  add("plain-date-time", "round", "2020-01-02T12:00", undefined, { smallestUnit: "day", roundingMode: "halfEven" });
  add("zoned-date-time", "round", "2020-01-02T12:00+00:00[UTC]", undefined, { smallestUnit: "day", roundingMode: "halfEven" });
  // A day rounding increment with largestUnit "week" applies to the days
  // left over after whole weeks.
  for (const roundingIncrement of [2, 3, 5, 10]) {
    for (const roundingMode of ["ceil", "trunc", "halfExpand", "halfEven"]) {
      add("plain-date", "until", "2006-08-24", ["2021-03-01"], { largestUnit: "week", smallestUnit: "day", roundingMode, roundingIncrement });
      add("plain-date", "since", "2006-08-24", ["2021-03-01"], { largestUnit: "week", smallestUnit: "day", roundingMode, roundingIncrement });
      add("plain-date-time", "until", "2020-01-01T00:00", ["2020-03-05T18:00"], { largestUnit: "week", smallestUnit: "day", roundingMode, roundingIncrement });
      add("zoned-date-time", "until", "2019-03-01T00:00-05:00[America/New_York]", ["2019-03-20T18:00-04:00[America/New_York]"],
        { largestUnit: "week", smallestUnit: "day", roundingMode, roundingIncrement });
    }
  }
  // Duration.prototype.add/subtract and toString() options.
  add("duration", "add", "PT59.999999999S", ["PT0.000000001S"]);
  add("duration", "subtract", "P1D", ["PT36H"]);
  add("duration", "add", "P1M", ["P1D"]);
  for (const options of [{ fractionalSecondDigits: 0 }, { fractionalSecondDigits: 2, roundingMode: "ceil" },
    { smallestUnit: "second", roundingMode: "halfExpand" }, { smallestUnit: "minute" }]) {
    for (const d of ["PT0S", "P1DT23H59M59.9S", "-PT1.25S", "P1Y2M", "PT36H", "P1DT36H"]) {
      add("duration", "toString", d, undefined, options);
    }
  }
  // Invalid option values are RangeErrors.
  add("plain-time", "toString", "12:00", undefined, { smallestUnit: "hour" });
  add("plain-date", "until", "2020-01-01", ["2021-01-01"], { roundingMode: "nearest" });
  add("zoned-date-time", "from", "2020-01-01T12:00+01:00[Europe/Paris]", undefined, { disambiguation: "first" });
  return out.map((c) => {
    const r = { ...c };
    for (const k of ["receiver", "args", "options"]) if (r[k] === undefined) delete r[k];
    return r;
  });
}
