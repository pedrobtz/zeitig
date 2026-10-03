# Add or subtract durations

`temporal_add()` and `temporal_subtract()` are Temporal's `.add()` and
`.subtract()`. The `+` and `-` operators do the same with the default
`overflow = "constrain"`.

## Usage

``` r
temporal_add(x, duration, ..., overflow = c("constrain", "reject"))

temporal_subtract(x, duration, ..., overflow = c("constrain", "reject"))
```

## Arguments

- x:

  A plain date, plain time, plain date-time, instant or zoned date-time.

- duration:

  A duration (or ISO 8601 duration string), recycled against `x`.

- ...:

  These dots are for future extensions and must be empty.

- overflow:

  `"constrain"` (default) clamps the day of month when adding years or
  months; `"reject"` raises an error instead.

## Value

An object of the same class as `x`.

## Details

- Plain dates: years and months are added first, clamping the day to the
  end of the month (`2021-01-31` + 1 month is `2021-02-28`), then weeks
  and days; time units are balanced into whole 24-hour days and any
  remainder is ignored.

- Plain times: only hours and smaller units count and the result wraps
  around midnight.

- Plain date-times: calendar units as for dates, then the time.

- Instants: only hours and smaller units are allowed (exact time).

- Zoned date-times: calendar units (years to days) follow the wall clock
  and are DST-aware (adding one day keeps the local time even when the
  day has 23 or 25 hours); time units are exact elapsed time.

Subtracting two values of the same type (`x - y`) is
`temporal_since(x, y)` with default options, giving a duration.

## See also

Other arithmetic:
[`temporal_round()`](https://pedrobtz.github.io/zudate/reference/temporal_round.md),
[`temporal_until()`](https://pedrobtz.github.io/zudate/reference/temporal_until.md)

## Examples

``` r
d <- plain_date(2021, 1, 31)
temporal_add(d, duration(months = 1))
#> <plain_date[1]>
#> [1] 2021-02-28
try(temporal_add(d, "P1M", overflow = "reject"))
#> Error in temporal_add(d, "P1M", overflow = "reject") : 
#>   day 31 is out of range for 2021-02 (overflow = "reject") (element 1)
d + duration(days = 1:3)
#> <plain_date[3]>
#> [1] 2021-02-01 2021-02-02 2021-02-03
plain_time(23, 30) + duration(hours = 1)
#> <plain_time[1]>
#> [1] 00:30:00
plain_date_time("2020-02-29T12:00") - duration(years = 1)
#> <plain_date_time[1]>
#> [1] 2019-02-28T12:00:00
```
