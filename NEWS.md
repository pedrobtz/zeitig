# zudate (development version)

* Rust dependencies (`jiff`, `savvy`) are vendored and the package builds
  offline from the source tarball.
* New `plain_date()`, `plain_time()` and `plain_date_time()` classes
  (Temporal `PlainDate`, `PlainTime`, `PlainDateTime`) with RFC 9557 parsing,
  `overflow` handling, field accessors (`year()`, ..., `nanosecond()`,
  `day_of_week()`, `week_of_year()`, ...), `temporal_with()`,
  `with_plain_time()`, `with_plain_date()`, `to_plain_*()` conversions,
  `temporal_compare()`/`temporal_equals()` and conversions to and from `Date`,
  `POSIXct` and `POSIXlt`.
* New `available_time_zones()` lists the IANA time zone identifiers known to
  the time zone database.
