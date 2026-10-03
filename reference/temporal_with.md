# Replace fields

`temporal_with()` returns a copy of `x` with some fields replaced, the
equivalent of Temporal's `.with()`. `with_plain_time()` and
`with_plain_date()` replace the whole time or date part of a plain
date-time (`.withPlainTime()`, `.withPlainDate()`).

## Usage

``` r
temporal_with(x, ..., overflow = c("constrain", "reject"))

with_plain_time(x, time = NULL)

with_plain_date(x, date)
```

## Arguments

- x:

  A Temporal object.

- ...:

  Named fields to replace, recycled with `x` to a common length. Plain
  dates take `year`, `month`, `day`; plain times take `hour`, `minute`,
  `second`, `millisecond`, `microsecond`, `nanosecond`; plain date-times
  take both.

- overflow:

  How to handle out-of-range values: `"constrain"` (the default) clamps,
  `"reject"` raises an error.

- time:

  A plain time (or string), recycled against `x`; `NULL` means midnight.

- date:

  A plain date (or string), recycled against `x`.

## Value

An object of the same class as `x`.

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
```
