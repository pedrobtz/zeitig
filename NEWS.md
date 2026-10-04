# zeitig 0.1.0

Initial CRAN release.

* Rust dependencies (`jiff`, `savvy`) are vendored and the package builds
  offline from the source tarball.
* New `plain_date()`, `plain_time()` and `plain_date_time()` classes
  (Temporal `PlainDate`, `PlainTime`, `PlainDateTime`) with RFC 9557 parsing,
  `overflow` handling, field accessors (`year()`, ..., `nanosecond()`,
  `day_of_week()`, `week_of_year()`, ...), `temporal_with()`,
  `with_plain_time()`, `with_plain_date()`, `to_plain_*()` conversions,
  `temporal_compare()`/`temporal_equals()` and conversions to and from `Date`,
  `POSIXct` and `POSIXlt`.
* New `duration()` class (Temporal `Duration`, backed by `jiff::Span`) with
  ISO 8601 parsing and formatting, `abs()`, `sign()`, unary `-`, `+`/`-`
  between durations, `duration_total()`, `duration_compare()`,
  `duration_blank()`, `as_duration()`/`as_difftime()`.
* Arithmetic for plain types: `temporal_add()`, `temporal_subtract()`,
  `+`/`-` operators, `temporal_until()`, `temporal_since()` and
  `temporal_round()` (also for durations, with `relative_to`).
* New `instant()` (Temporal `Instant`) and `zoned_date_time()` (Temporal
  `ZonedDateTime`) classes: RFC 9557 parsing with `disambiguation` and
  `offset` options, per-element time zones, `epoch_*()` accessors,
  `time_zone()`, `offset()`, `offset_nanoseconds()`, `hours_in_day()`,
  `start_of_day()`, `time_zone_transition()`, `with_time_zone()`,
  `to_instant()`, `to_zoned_date_time()`, DST-aware arithmetic, rounding and
  differences, and `POSIXct`/`POSIXlt` conversions.
* `now_instant()`, `now_zoned_date_time()`, `now_plain_date()`,
  `now_plain_time()`, `now_plain_date_time()` and `now_time_zone()`.
* `format()` supports Temporal's `toString()` options
  (`fractional_second_digits`, `smallest_unit`, `rounding_mode`, `offset`,
  `time_zone_name`, `calendar_name`, `time_zone`), and
  `temporal_strftime()`/`temporal_strptime()` expose jiff's strftime.
* `summary()` methods; vignettes `vignette("zeitig")`,
  `vignette("time-zones")` and `vignette("temporal-differences")`.
* New `available_time_zones()` lists the IANA time zone identifiers known to
  the time zone database. A `ZEITIG_TZDIR` environment variable that does not
  point at a time zone database is an error rather than silently ignored.
* `temporal_with()` also replaces the fields of durations
  (`Duration.prototype.with()`).
* Strings combine with and compare against every zeitig class by parsing, as
  they do with base R's `Date`: `plain_date(2020, 1, 1) == "2020-01-01"`.
* Error messages never name jiff's Rust types; for example a duration with
  months used without `relative_to` reports that it "requires `relative_to`".
* A conformance test compares about 6,300 operations with the reference
  Temporal implementation, including a selection of test262 cases. Every
  remaining difference is listed in `vignette("temporal-differences")`.
  Behaviour it checks includes:
  * `[!u-ca=iso8601]` annotations are accepted. Other calendar annotations
    are an error.
  * UTC offsets of 24 hours or more are an error. So are offsets with
    fractional seconds for instants and zoned date-times.
  * `temporal_since()` with calendar units, the `halfEven` rounding mode,
    day rounding increments with `largest_unit = "week"`, and the rounding
    of instants before 1970 give Temporal's results, where 'jiff' alone
    would differ.
  * A `+00:00` time zone is kept apart from `UTC`. `temporal_equals()`
    treats `UTC`, `Etc/UTC`, `Etc/GMT` and `GMT` as the same zone.
  * `instant_from_epoch()` gives an error for out-of-range nanoseconds.
  * `temporal_add()` and `temporal_subtract()` accept durations, and
    `format()` on durations supports `fractional_second_digits`,
    `smallest_unit` and `rounding_mode`.
  * Invalid option values raise `zeitig_range_error`, as Temporal raises a
    `RangeError`.
