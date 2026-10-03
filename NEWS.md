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
  the time zone database.
