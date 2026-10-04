# Replace fields

`temporal_with()` returns a copy of `x` with some fields replaced, the
equivalent of Temporal's `.with()` (including
`Duration.prototype.with()`). `with_plain_time()` and
`with_plain_date()` replace the whole time or date part of a plain or
zoned date-time (`.withPlainTime()`, `.withPlainDate()`).

## Usage

``` r
temporal_with(
  x,
  ...,
  overflow = c("constrain", "reject"),
  disambiguation = c("compatible", "earlier", "later", "reject"),
  offset = c("prefer", "use", "ignore", "reject")
)

with_plain_time(x, time = NULL)

with_plain_date(x, date)
```

## Arguments

- x:

  A Temporal object.

- ...:

  Named fields to replace, recycled with `x` to a common length. Plain
  dates take `year`, `month`, `day`; plain times take `hour`, `minute`,
  `second`, `millisecond`, `microsecond`, `nanosecond`; plain and zoned
  date-times take both; durations take `years` to `nanoseconds` (the
  result must still have fields of one sign).

- overflow:

  How to handle out-of-range values: `"constrain"` (the default) clamps,
  `"reject"` raises an error. Not used for durations.

- disambiguation, offset:

  Zoned date-times only; see
  [`zoned_date_time()`](https://pedrobtz.github.io/zeitig/reference/zoned_date_time.md).
  `offset` defaults to `"prefer"` here.

- time:

  A plain time (or string), recycled against `x`; `NULL` means midnight.

- date:

  A plain date (or string), recycled against `x`.

## Value

An object of the same class as `x`.

## Details

For zoned date-times the fields are wall-clock fields in the element's
time zone; the new local time is resolved with `disambiguation`, and
`offset = "prefer"` (the default) keeps the current UTC offset when it
is still valid, so that changing a field inside a DST overlap stays on
the same side of it. `with_plain_time(x)` without `time` gives the start
of the day.

## Examples

``` r
d <- plain_date(2006, 1, 24)
temporal_with(d, day = 31, month = 2) # 2006-02-28
#> <plain_date[1]>
#> [1] 2006-02-28
dt <- plain_date_time("1995-12-07T03:24:30")
temporal_with(dt, hour = 12)
#> <plain_date_time[1]>
#> [1] 1995-12-07T12:24:30
with_plain_time(dt, plain_time(9, 30))
#> <plain_date_time[1]>
#> [1] 1995-12-07T09:30:00
with_plain_time(dt) # midnight
#> <plain_date_time[1]>
#> [1] 1995-12-07T00:00:00
with_plain_date(dt, plain_date(2000, 1, 1))
#> <plain_date_time[1]>
#> [1] 2000-01-01T03:24:30
z <- zoned_date_time("2019-11-03T01:30-04:00[America/New_York]")
temporal_with(z, minute = 45) # stays at -04:00 in the DST overlap
#> <zoned_date_time[1]>
#> [1] 2019-11-03T01:45:00-04:00[America/New_York]
with_plain_time(z)
#> <zoned_date_time[1]>
#> [1] 2019-11-03T00:00:00-04:00[America/New_York]
temporal_with(duration(hours = 1, minutes = 30), minutes = 0)
#> <duration[1]>
#> [1] PT1H
```
