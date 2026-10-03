# Durations

A `zeitig_duration` is a length of time expressed in calendar and clock
units, the equivalent of
[`Temporal.Duration`](https://tc39.es/proposal-temporal/docs/duration.html)
(and of `jiff::Span`). Each element stores ten integer fields that must
all have the same sign; they are not balanced, so
`duration(minutes = 90)` stays 90 minutes.

## Usage

``` r
duration(
  years = 0,
  months = 0,
  weeks = 0,
  days = 0,
  hours = 0,
  minutes = 0,
  seconds = 0,
  milliseconds = 0,
  microseconds = 0,
  nanoseconds = 0,
  ...
)

is_duration(x)
```

## Arguments

- years, months, weeks, days, hours, minutes, seconds, milliseconds,
  microseconds, nanoseconds:

  Integer-valued numbers (stored as doubles, so values beyond 2^53 lose
  precision). `years` may instead be a character vector to parse.

- ...:

  These dots are for future extensions and must be empty.

- x:

  An object to test.

## Value

A `zeitig_duration` vector.

## Details

`duration()` builds durations from fields (recycled to a common length,
all defaulting to zero) or, when given a single character vector, parses
ISO 8601 duration strings such as `"P1Y2M3DT4H5M6.5S"`. Use
[`temporal_fields()`](https://pedrobtz.github.io/zeitig/reference/temporal-fields.md)
(or [`vctrs::field()`](https://vctrs.r-lib.org/reference/fields.html))
to read the fields.

Durations support unary `-`,
[`abs()`](https://rdrr.io/r/base/MathFun.html),
[`sign()`](https://rdrr.io/r/base/sign.html), `+` and `-` between
durations (calendar units need
[`temporal_add()`](https://pedrobtz.github.io/zeitig/reference/temporal_add.md)
on a date instead), and arithmetic with dates and times (see
[`temporal_add()`](https://pedrobtz.github.io/zeitig/reference/temporal_add.md)).
Comparison operators order durations by their length with 24-hour days;
durations with years, months or weeks can only be compared with
[`duration_compare()`](https://pedrobtz.github.io/zeitig/reference/duration_total.md)
and a `relative_to` date. `==` compares the fields, so `PT1H == PT60M`
is `FALSE` (use
[`duration_compare()`](https://pedrobtz.github.io/zeitig/reference/duration_total.md)
to compare lengths).

## See also

Other duration:
[`as_duration()`](https://pedrobtz.github.io/zeitig/reference/as_duration.md),
[`duration_total()`](https://pedrobtz.github.io/zeitig/reference/duration_total.md)

## Examples

``` r
duration(hours = 1, minutes = 30)
#> <duration[1]>
#> [1] PT1H30M
duration(c("P1Y2M3DT4H", "-PT1.5S", "PT0S", NA))
#> <duration[4]>
#> [1] P1Y2M3DT4H -PT1.5S    PT0S       <NA>      
-duration(days = 3)
#> <duration[1]>
#> [1] -P3D
abs(duration("-P1D"))
#> <duration[1]>
#> [1] P1D
duration(hours = 1) + duration(minutes = 90)
#> <duration[1]>
#> [1] PT2H30M
sort(duration(c("PT2H", "PT90M", "P1D")))
#> <duration[3]>
#> [1] PT90M PT2H  P1D  
```
