# Duration helpers

- `duration_total()` expresses a duration as a (fractional) number of
  `unit`s (`Duration.prototype.total()`).

- `duration_compare()` compares two durations by length, returning -1, 0
  or 1 (`Duration.compare()`).

- `duration_blank()` is `TRUE` for zero durations (`.blank`).

## Usage

``` r
duration_total(x, unit, relative_to = NULL)

duration_compare(x, y, relative_to = NULL)

duration_blank(x)
```

## Arguments

- x, y:

  Durations (or ISO 8601 strings), recycled to a common length.

- unit:

  The unit to express the duration in, e.g. `"hour"`. Plural spellings
  (`"hours"`) are accepted.

- relative_to:

  `NULL`, or a plain date, plain date-time or zoned date-time (or
  string) recycled against `x`.

## Value

`duration_total()`: a double vector. `duration_compare()`: an integer
vector. `duration_blank()`: a logical vector.

## Details

Without `relative_to`, days are 24 hours long and durations with years,
months are an error. With `relative_to` (a plain date, plain date-time
or zoned date-time, or a string), calendar units are resolved from that
starting point; with a zoned date-time, days follow the time zone's DST
rules.

## See also

Other duration:
[`as_duration()`](https://pedrobtz.github.io/zudate/reference/as_duration.md),
[`duration()`](https://pedrobtz.github.io/zudate/reference/duration.md)

## Examples

``` r
duration_total(duration(hours = 1, minutes = 30), "minute")
#> [1] 90
duration_total(duration(months = 1), "day", relative_to = plain_date(2020, 2, 1))
#> [1] 29
duration_compare(duration(hours = 1), duration(minutes = 60))
#> [1] 0
duration_compare(duration(months = 1), duration(days = 30), relative_to = "2020-02-01")
#> [1] -1
duration_blank(duration(c("PT0S", "PT1S")))
#> [1]  TRUE FALSE
```
