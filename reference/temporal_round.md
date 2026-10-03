# Round values and durations

`temporal_round()` is Temporal's `.round()`.

## Usage

``` r
temporal_round(
  x,
  smallest_unit = NULL,
  ...,
  rounding_increment = 1,
  rounding_mode = "halfExpand",
  largest_unit = NULL,
  relative_to = NULL
)
```

## Arguments

- x:

  A plain time, plain date-time, instant, zoned date-time or duration.

- smallest_unit:

  The unit to round to (plurals accepted). For durations the default is
  `"nanosecond"`.

- ...:

  These dots are for future extensions and must be empty.

- rounding_increment:

  Round to multiples of this many `smallest_unit`s.

- rounding_mode:

  Defaults to `"halfExpand"` (round half away from zero); see
  [`temporal_until()`](https://pedrobtz.github.io/zudate/reference/temporal_until.md)
  for the other modes.

- largest_unit:

  Durations only: the largest unit of the result (`"auto"` keeps the
  largest non-zero unit of `x`).

- relative_to:

  Durations only: `NULL`, or a plain date, plain date-time or zoned
  date-time (or string) recycled against `x`.

## Value

An object of the same class as `x`.

## Details

- Plain times, plain date-times, instants and zoned date-times round to
  `smallest_unit` (at most `"hour"` for times and instants, `"day"` for
  date-times; a zoned day may be 23 or 25 hours long).
  `rounding_increment` must divide evenly into the next larger unit (for
  instants, into a day).

- Durations are rounded and balanced between `largest_unit` and
  `smallest_unit`. Without `relative_to`, days are 24 hours and years,
  months and weeks are an error; with it, calendar units are resolved
  from that date.

## See also

Other arithmetic:
[`temporal_add()`](https://pedrobtz.github.io/zudate/reference/temporal_add.md),
[`temporal_until()`](https://pedrobtz.github.io/zudate/reference/temporal_until.md)

## Examples

``` r
temporal_round(plain_time(19, 39, 9, 68, 346, 205), "hour")
#> <plain_time[1]>
#> [1] 20:00:00
temporal_round(plain_time(19, 39, 9), "minute", rounding_increment = 15, rounding_mode = "floor")
#> <plain_time[1]>
#> [1] 19:30:00
temporal_round(plain_date_time("1995-12-07T03:24:30.000003500"), "second")
#> <plain_date_time[1]>
#> [1] 1995-12-07T03:24:30
temporal_round(duration(minutes = 130), largest_unit = "hour")
#> <duration[1]>
#> [1] PT2H10M
temporal_round(duration(days = 45), largest_unit = "month", relative_to = "2020-01-01")
#> <duration[1]>
#> [1] P1M14D
```
