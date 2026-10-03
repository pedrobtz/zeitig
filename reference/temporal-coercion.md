# Coerce to Temporal types

S3 generics that convert other objects, including base R classes, to
Temporal types. Character vectors are parsed as RFC 9557 / ISO 8601
strings.

## Usage

``` r
as_plain_date(x, ...)

as_plain_time(x, ...)

as_plain_date_time(x, ...)

as_instant(x, ...)

as_zoned_date_time(x, ...)
```

## Arguments

- x:

  An object to convert.

- ...:

  Passed on to methods.

## Value

A Temporal vector of the requested type.

## Details

- `Date` converts exactly to a plain date (and to midnight for a plain
  date-time).

- `POSIXct` and `POSIXlt` convert to the wall-clock date and time in
  their own time zone (the `tzone` attribute, or the session time zone).
  `POSIXct` values are rounded to the microsecond, because a double
  cannot hold nanoseconds of a present-day instant; `POSIXlt` seconds
  are rounded to the nanosecond.

- `POSIXct` converts to an instant, or to a zoned date-time in its
  `tzone` (the session time zone, see
  [`now_time_zone()`](https://pedrobtz.github.io/zeitig/reference/now_instant.md),
  when unset); `as_zoned_date_time(x, time_zone = )` picks another zone.

The reverse conversions use base generics:
[`as.Date()`](https://rdrr.io/r/base/as.Date.html) for plain dates,
plain date-times and zoned date-times (wall-clock date), and
[`as.POSIXct()`](https://rdrr.io/r/base/as.POSIXlt.html) /
[`as.POSIXlt()`](https://rdrr.io/r/base/as.POSIXlt.html). For plain
date-times these interpret the wall-clock time in `tz` (UTC by default;
local times in DST gaps are resolved by the operating system). Instants
and zoned date-times convert exactly up to the microsecond; zoned
date-times keep their time zone when all elements share one IANA zone
and `tz` is not given, otherwise `tz` (default UTC) is used.

## Examples

``` r
as_plain_date(as.Date("2020-02-29"))
#> <plain_date[1]>
#> [1] 2020-02-29
as_plain_date_time(as.POSIXct("2020-02-29 12:30:00", tz = "UTC"))
#> <plain_date_time[1]>
#> [1] 2020-02-29T12:30:00
as_plain_time("12:30")
#> <plain_time[1]>
#> [1] 12:30:00
as.Date(plain_date(2020, 2, 29))
#> [1] "2020-02-29"
as.POSIXct(plain_date_time("2020-02-29T12:30"), tz = "America/New_York")
#> [1] "2020-02-29 12:30:00 EST"
as_instant(as.POSIXct("2020-01-01 12:00:00", tz = "UTC"))
#> <instant[1]>
#> [1] 2020-01-01T12:00:00Z
as_zoned_date_time(as.POSIXct("2020-07-01 12:00:00", tz = "Europe/Paris"))
#> <zoned_date_time[1]>
#> [1] 2020-07-01T12:00:00+02:00[Europe/Paris]
as.POSIXct(zoned_date_time("2020-07-01T12:00+02:00[Europe/Paris]"))
#> [1] "2020-07-01 12:00:00 CEST"
```
