# Changelog

## zudate (development version)

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
  [`available_time_zones()`](https://pedrobtz.github.io/zudate/reference/available_time_zones.md)
  lists the IANA time zone identifiers known to the time zone database.
