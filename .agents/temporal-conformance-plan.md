# Plan: testing zeitig against Temporal

Status: implemented. The figures quoted below were measured with `@js-temporal/polyfill` 0.5.1.

- `tools/temporal-oracle/`: `generate.mjs` (runner), `cases/*.mjs` (generators per type,
  `regressions.mjs` for each fixed difference, `test262.mjs` for 447 cases from 47 test262 files),
  `divergences.mjs` (the design.md section 9 keys), `api.mjs` + `api-map.csv` (API share),
  `report.R` (match share).
- `tests/testthat/helper-conformance.R` (dispatch table) and `test-temporal-conformance.R`.
- Result: 6,262 cases; 98.8% match Temporal, 1.2% are documented differences, 0 mismatches.
  161 of 216 Temporal members (75%) are implemented.
- Fixtures: about 1 MB of JSON on disk, about 75 KB compressed in the tarball. This is above the
  500 KB target below, which was chosen with the tarball in mind. The test takes about
  10 seconds. Most of that is per-call overhead: about 3,500 calls, because options are scalar
  and every expected error runs on its own.

## Goal

Measure how closely zeitig matches Temporal, and keep it that way:

- Run the same inputs through Temporal and through zeitig and compare the results, so semantic
  bugs are caught automatically. For example, `plain_date("2024-03-15[u-ca=hebrew]")` currently
  ignores the calendar annotation instead of raising `zeitig_range_error` as `design.md` section 9
  requires.
- Make every documented difference (`design.md` section 9, `vignettes/temporal-differences.Rmd`)
  a tested, expected mismatch. Any undocumented mismatch fails the test suite.
- Report a coverage figure: the share of generated cases that match, and the share of the Temporal
  API that zeitig implements.

## Why not port test262 wholesale

test262 (`test/built-ins/Temporal/`) has thousands of files, but most check JavaScript mechanics
that do not apply to an R package: prototype and property descriptors, argument-type coercion,
the order in which properties are read, the calendar protocol, `BigInt`. `intl402/Temporal` covers
locales and non-ISO calendars, which jiff does not support. Only a minority of the files check
date and time results.

## Approach

### 1. Compare against the reference implementation (main approach)

```
tools/temporal-oracle/            # excluded from the build (tools/ is in .Rbuildignore)
  package.json                    # pins @js-temporal/polyfill to an exact version
  package-lock.json
  generate.mjs                    # builds the cases, runs them through the polyfill
  cases/*.mjs                     # one case generator per area
tests/testthat/fixtures/temporal/
  plain-date.json ...             # generated, committed, small
tests/testthat/test-temporal-conformance.R
```

- Use the npm polyfill as the reference, not Node's built-in `Temporal`. Node 22's built-in one
  (`--harmony-temporal`) is an old, incomplete V8 implementation. Record the polyfill version in
  every fixture file.
- CRAN never needs Node: fixtures are committed, and the test only reads them.
- `generate.mjs` is deterministic (fixed seeds, sorted output), like `tools/vendor.sh`, so
  regenerating without changes gives an identical diff.

Fixture record (one per case):

```json
{
  "id": "plain-date/add/0042",
  "op": "add",
  "receiver": "2021-01-31",
  "args": ["P1M"],
  "options": {"overflow": "constrain"},
  "result": "2021-02-28",
  "error": null,
  "divergence": null
}
```

- Exchange everything as strings in Temporal's `toString()` format (RFC 9557 and ISO 8601
  durations). Both sides round-trip it, and no numeric precision is lost.
- `error` holds the JavaScript error class (`RangeError`, `TypeError`). zeitig must also raise an
  error; the condition class is mapped (`RangeError` -> `zeitig_range_error`) and the message is
  not compared.
- `divergence` names a row of `design.md` section 9 (for example `"instant-range"`). The test then
  expects zeitig's documented behaviour, or skips the case with that reason. A mismatch on a case
  with no `divergence` fails the test.
- The test maps each `op` to the zeitig function with a small dispatch table (`add` ->
  `temporal_add()`, `until` -> `temporal_until()`, option names camelCase -> snake_case).

Inputs to generate, per type:

- Parsing (`from`): valid and invalid RFC 9557 strings, annotations (`[u-ca=...]`, `[!Zone]`),
  fractional seconds, extended years, `overflow`.
- `add`/`subtract`: month-end overflow, leap years, mixed-sign durations, `overflow`.
- `until`/`since`: every `largest_unit` x `smallest_unit` x `rounding_mode` x `rounding_increment`
  combination.
- `round`: units, modes, increments; durations with and without `relative_to`.
- `with`: field changes, `overflow`, and `offset` for zoned values.
- Zoned values: DST gaps and overlaps in zones with unusual rules (`America/New_York`,
  `Europe/London`, `Australia/Lord_Howe` (30-minute DST), `Pacific/Apia` (skipped a day),
  `Asia/Kolkata`), every `disambiguation` and `offset` option.
- `toString`: `fractional_second_digits`, `smallest_unit`, `rounding_mode`, `offset`,
  `time_zone_name`, `calendar_name`.
- `compare`/`equals`, `total`.

Time zone rules change between tzdata releases. Generate fixtures only for dates that are stable
in the IANA database, or pin a date range, so that tests do not depend on the machine's tzdata.

### 2. Port a curated slice of test262 (supplement)

Port only result-checking tests:

- `test/staging/Temporal/` (scenario tests, mostly results).
- `test/built-ins/Temporal/*/prototype/{add,subtract,until,since,round,with,toString}/` and
  `*/from/`, files whose checks call `TemporalHelpers.assert*` or `assert.sameValue` on a result.

Skip property descriptors, `prop-desc`, `builtin`, `length`, `name`, coercion (`*-wrong-type`,
`*-not-object`), observable property-access order (`order-of-operations`), the calendar protocol,
and `intl402/`.

test262 is BSD-3-Clause, owned by Ecma International. Ported cases go into the same fixture format
with `"source": "test262/<path>"`. If any ship in the tarball, add the test262 copyright and
license to `LICENSE.note`. Keep the ported set small; the comparison harness is the main approach.

## Order of work

1. Harness for `PlainDate` only (`from`, `add`, `until`, `round`, `with`, `toString`): settle the
   fixture format, the dispatch table and the `divergence` mechanism.
2. Fix or document whatever it finds; start with the ignored `[u-ca=...]` annotation.
3. Extend to `PlainTime`, `PlainDateTime`, `Duration`, then `Instant` and `ZonedDateTime`.
4. Add the curated test262 slice.
5. Publish the figures (cases matched, API share implemented) in
   `vignettes/temporal-differences.Rmd` and keep `design.md` section 9 in sync.

## Constraints

- Tarball: fixtures stay small. CI enforces 5 MB for the whole tarball, and `vendor.tar.xz` already
  takes about 1.5 MB. Aim for under 500 KB of fixtures; prefer covering combinations over volume.
- Test time: the conformance test should run in seconds. Load each fixture file once and call
  zeitig vectorised over all cases of an `op`, not once per case.
- Dependencies: reading JSON needs `jsonlite` in `Suggests`, with `skip_if_not_installed()`.
  Alternatively, write fixtures as tab-separated text and read them with base R.
- Never test against a live network or a live Node install in `tests/`.

## Decisions (formerly open questions)

- JSON, read with `zujson` (the author's own CRAN package) in `Suggests`, because the cases have
  nested arguments and options.
- No scheduled regeneration job yet. Fixtures change only when someone runs `generate.mjs`.
  The zoned cases avoid tzdata drift by using 2000-2024 dates and no link names.
- Both figures are reported. The API share counts static and prototype members of the
  polyfill's classes that have a zeitig equivalent in `api-map.csv`, leaving out `valueOf`.
  The case share counts generated cases that match.
