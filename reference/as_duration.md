# Coerce to and from durations

`as_duration()` converts ISO 8601 strings and `difftime` objects to
durations; `as_difftime()` converts durations without calendar units
(years, months, weeks) to `difftime`, treating days as 24 hours.

## Usage

``` r
as_duration(x, ...)

as_difftime(x, ..., units = "auto")
```

## Arguments

- x:

  An object to convert.

- ...:

  Passed on to methods.

- units:

  The units of the result, as in
  [`base::difftime()`](https://rdrr.io/r/base/difftime.html).

## Value

A duration (`as_duration()`) or a `difftime` (`as_difftime()`).

## Details

A `difftime` in days, hours, minutes or seconds keeps that unit when its
values are whole numbers; otherwise (and for weeks) it becomes seconds
plus a sub-second part rounded to the nanosecond.

## See also

Other duration:
[`duration()`](https://pedrobtz.github.io/zudate/reference/duration.md),
[`duration_total()`](https://pedrobtz.github.io/zudate/reference/duration_total.md)

## Examples

``` r
as_duration(as.difftime(90, units = "mins"))
#> <duration[1]>
#> [1] PT90M
as_duration(as.difftime(1.5, units = "hours"))
#> <duration[1]>
#> [1] PT5400S
as_difftime(duration(hours = 1, minutes = 30))
#> Time difference of 1.5 hours
as_difftime(duration(hours = 36), units = "days")
#> Time difference of 1.5 days
```
