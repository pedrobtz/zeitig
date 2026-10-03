# Zoned date-times

A `zeitig_zoned_date_time` is an exact time together with the time zone
used to view it, the equivalent of
[`Temporal.ZonedDateTime`](https://tc39.es/proposal-temporal/docs/zoneddatetime.html)
(and of `jiff::Zoned`). Each element carries its own time zone: an IANA
identifier (see
[`available_time_zones()`](https://pedrobtz.github.io/zeitig/reference/available_time_zones.md))
or a fixed offset such as `"+05:30"`. Arithmetic with calendar units
follows the wall clock and is DST-aware.

## Usage

``` r
zoned_date_time(
  year,
  month,
  day,
  hour = 0L,
  minute = 0L,
  second = 0L,
  millisecond = 0L,
  microsecond = 0L,
  nanosecond = 0L,
  ...,
  time_zone,
  disambiguation = c("compatible", "earlier", "later", "reject"),
  offset = c("reject", "use", "prefer", "ignore"),
  overflow = c("constrain", "reject")
)

is_zoned_date_time(x)
```

## Arguments

- year, month, day:

  Integer-valued numbers, or for `year` a character vector to parse.

- hour, minute, second, millisecond, microsecond, nanosecond:

  Integer-valued numbers. Fractions are truncated towards zero. `hour`
  may instead be a character vector to parse.

- ...:

  These dots are for future extensions and must be empty.

- time_zone:

  Time zone identifiers, recycled against the components.

- disambiguation:

  For local times that occur twice (overlaps) or not at all (gaps):
  `"compatible"` (default; the earlier time in an overlap, the later in
  a gap), `"earlier"`, `"later"` or `"reject"`.

- offset:

  When parsing: `"reject"` (default) errors if the offset in the string
  is not valid for the time zone, `"use"` keeps the exact time given by
  the offset, `"ignore"` keeps the wall-clock time, `"prefer"` uses the
  offset when it is valid and the wall-clock time otherwise.

- overflow:

  How to handle out-of-range values: `"constrain"` (the default) clamps
  each component to its range, `"reject"` raises an error.

- x:

  An object to test.

## Value

A `zeitig_zoned_date_time` vector.

## Details

`zoned_date_time()` builds values from wall-clock components and a time
zone, resolving local times that fall in a DST gap or overlap with
`disambiguation`. Given a single character vector it parses RFC 9557
strings, which must carry a time zone annotation such as
`"2020-03-08T03:30-04:00[America/New_York]"`; `offset` says what to do
when the string's UTC offset disagrees with the time zone.

## See also

Other zoned date-time:
[`time_zone()`](https://pedrobtz.github.io/zeitig/reference/time_zone.md)

## Examples

``` r
zoned_date_time(1995, 12, 7, 3, 24, 30, time_zone = "America/New_York")
#> <zoned_date_time[1]>
#> [1] 1995-12-07T03:24:30-05:00[America/New_York]
zoned_date_time("1995-12-07T03:24:30-08:00[America/Los_Angeles]")
#> <zoned_date_time[1]>
#> [1] 1995-12-07T03:24:30-08:00[America/Los_Angeles]
# 02:30 does not exist on 2019-03-10 in New York
zoned_date_time(2019, 3, 10, 2, 30, time_zone = "America/New_York")
#> <zoned_date_time[1]>
#> [1] 2019-03-10T03:30:00-04:00[America/New_York]
zoned_date_time(2019, 3, 10, 2, 30, time_zone = "America/New_York", disambiguation = "earlier")
#> <zoned_date_time[1]>
#> [1] 2019-03-10T01:30:00-05:00[America/New_York]
zoned_date_time(2020, 1, 1, time_zone = c("UTC", "Asia/Tokyo", "+05:30"))
#> <zoned_date_time[3]>
#> [1] 2020-01-01T00:00:00+00:00[UTC]        2020-01-01T00:00:00+09:00[Asia/Tokyo]
#> [3] 2020-01-01T00:00:00+05:30[+05:30]    
```
