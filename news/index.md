# Changelog

## zudate 0.1.0

Initial CRAN release.

- Rust dependencies (`jiff`, `savvy`) are vendored and the package
  builds offline from the source tarball.
- New
  [`plain_date()`](https://pedrobtz.github.io/zudate/reference/plain_date.md),
  [`plain_time()`](https://pedrobtz.github.io/zudate/reference/plain_time.md)
  and
  [`plain_date_time()`](https://pedrobtz.github.io/zudate/reference/plain_date_time.md)
  classes (Temporal `PlainDate`, `PlainTime`, `PlainDateTime`) with RFC
  9557 parsing, `overflow` handling, field accessors
  ([`year()`](https://pedrobtz.github.io/zudate/reference/temporal-fields.md),
  …,
  [`nanosecond()`](https://pedrobtz.github.io/zudate/reference/temporal-fields.md),
  [`day_of_week()`](https://pedrobtz.github.io/zudate/reference/temporal-fields.md),
  [`week_of_year()`](https://pedrobtz.github.io/zudate/reference/temporal-fields.md),
  …),
  [`temporal_with()`](https://pedrobtz.github.io/zudate/reference/temporal_with.md),
  [`with_plain_time()`](https://pedrobtz.github.io/zudate/reference/temporal_with.md),
  [`with_plain_date()`](https://pedrobtz.github.io/zudate/reference/temporal_with.md),
  `to_plain_*()` conversions,
  [`temporal_compare()`](https://pedrobtz.github.io/zudate/reference/temporal_compare.md)/[`temporal_equals()`](https://pedrobtz.github.io/zudate/reference/temporal_compare.md)
  and conversions to and from `Date`, `POSIXct` and `POSIXlt`.
- New
  [`duration()`](https://pedrobtz.github.io/zudate/reference/duration.md)
  class (Temporal `Duration`, backed by `jiff::Span`) with ISO 8601
  parsing and formatting,
  [`abs()`](https://rdrr.io/r/base/MathFun.html),
  [`sign()`](https://rdrr.io/r/base/sign.html), unary `-`, `+`/`-`
  between durations,
  [`duration_total()`](https://pedrobtz.github.io/zudate/reference/duration_total.md),
  [`duration_compare()`](https://pedrobtz.github.io/zudate/reference/duration_total.md),
  [`duration_blank()`](https://pedrobtz.github.io/zudate/reference/duration_total.md),
  [`as_duration()`](https://pedrobtz.github.io/zudate/reference/as_duration.md)/[`as_difftime()`](https://pedrobtz.github.io/zudate/reference/as_duration.md).
- Arithmetic for plain types:
  [`temporal_add()`](https://pedrobtz.github.io/zudate/reference/temporal_add.md),
  [`temporal_subtract()`](https://pedrobtz.github.io/zudate/reference/temporal_add.md),
  `+`/`-` operators,
  [`temporal_until()`](https://pedrobtz.github.io/zudate/reference/temporal_until.md),
  [`temporal_since()`](https://pedrobtz.github.io/zudate/reference/temporal_until.md)
  and
  [`temporal_round()`](https://pedrobtz.github.io/zudate/reference/temporal_round.md)
  (also for durations, with `relative_to`).
- New
  [`instant()`](https://pedrobtz.github.io/zudate/reference/instant.md)
  (Temporal `Instant`) and
  [`zoned_date_time()`](https://pedrobtz.github.io/zudate/reference/zoned_date_time.md)
  (Temporal `ZonedDateTime`) classes: RFC 9557 parsing with
  `disambiguation` and `offset` options, per-element time zones,
  `epoch_*()` accessors,
  [`time_zone()`](https://pedrobtz.github.io/zudate/reference/time_zone.md),
  [`offset()`](https://pedrobtz.github.io/zudate/reference/time_zone.md),
  [`offset_nanoseconds()`](https://pedrobtz.github.io/zudate/reference/time_zone.md),
  [`hours_in_day()`](https://pedrobtz.github.io/zudate/reference/time_zone.md),
  [`start_of_day()`](https://pedrobtz.github.io/zudate/reference/time_zone.md),
  [`time_zone_transition()`](https://pedrobtz.github.io/zudate/reference/time_zone.md),
  [`with_time_zone()`](https://pedrobtz.github.io/zudate/reference/time_zone.md),
  [`to_instant()`](https://pedrobtz.github.io/zudate/reference/temporal-conversions.md),
  [`to_zoned_date_time()`](https://pedrobtz.github.io/zudate/reference/temporal-conversions.md),
  DST-aware arithmetic, rounding and differences, and
  `POSIXct`/`POSIXlt` conversions.
- [`now_instant()`](https://pedrobtz.github.io/zudate/reference/now_instant.md),
  [`now_zoned_date_time()`](https://pedrobtz.github.io/zudate/reference/now_instant.md),
  [`now_plain_date()`](https://pedrobtz.github.io/zudate/reference/now_instant.md),
  [`now_plain_time()`](https://pedrobtz.github.io/zudate/reference/now_instant.md),
  [`now_plain_date_time()`](https://pedrobtz.github.io/zudate/reference/now_instant.md)
  and
  [`now_time_zone()`](https://pedrobtz.github.io/zudate/reference/now_instant.md).
- [`format()`](https://rdrr.io/r/base/format.html) supports Temporal’s
  [`toString()`](https://rdrr.io/r/base/toString.html) options
  (`fractional_second_digits`, `smallest_unit`, `rounding_mode`,
  `offset`, `time_zone_name`, `calendar_name`, `time_zone`), and
  [`temporal_strftime()`](https://pedrobtz.github.io/zudate/reference/temporal_strftime.md)/[`temporal_strptime()`](https://pedrobtz.github.io/zudate/reference/temporal_strftime.md)
  expose jiff’s strftime.
- [`summary()`](https://rdrr.io/r/base/summary.html) methods; vignettes
  [`vignette("zudate")`](https://pedrobtz.github.io/zudate/articles/zudate.md)
  and
  [`vignette("time-zones")`](https://pedrobtz.github.io/zudate/articles/time-zones.md).
- New
  [`available_time_zones()`](https://pedrobtz.github.io/zudate/reference/available_time_zones.md)
  lists the IANA time zone identifiers known to the time zone database.
