# Difference between two values

`temporal_until(x, y)` is the duration from `x` to `y` (Temporal's
`x.until(y)`); `temporal_since(x, y)` is the duration from `y` to `x`
(`x.since(y)`), i.e. the negation of `temporal_until(x, y)`.

## Usage

``` r
temporal_until(
  x,
  y,
  ...,
  largest_unit = "auto",
  smallest_unit = NULL,
  rounding_increment = 1,
  rounding_mode = "trunc"
)

temporal_since(
  x,
  y,
  ...,
  largest_unit = "auto",
  smallest_unit = NULL,
  rounding_increment = 1,
  rounding_mode = "trunc"
)
```

## Arguments

- x, y:

  Values of the same type (or strings parsed as that type), recycled to
  a common length.

- ...:

  These dots are for future extensions and must be empty.

- largest_unit, smallest_unit:

  Units such as `"year"`, `"day"` or `"millisecond"` (plurals accepted);
  `largest_unit` may be `"auto"` and `smallest_unit` `NULL` for the
  type's default.

- rounding_increment:

  Round to multiples of this many `smallest_unit`s.

- rounding_mode:

  One of `"trunc"` (default), `"ceil"`, `"floor"`, `"expand"`,
  `"halfCeil"`, `"halfFloor"`, `"halfExpand"`, `"halfTrunc"`,
  `"halfEven"`.

## Value

A duration vector.

## Details

The result is balanced up to `largest_unit` and rounded to
`smallest_unit`. The default largest unit (`"auto"`) is `"day"` for
plain dates and plain date-times and `"hour"` for plain times; the
default smallest unit is `"day"` for plain dates and `"nanosecond"`
otherwise.

## See also

Other arithmetic:
[`temporal_add()`](https://pedrobtz.github.io/zudate/reference/temporal_add.md),
[`temporal_round()`](https://pedrobtz.github.io/zudate/reference/temporal_round.md)

## Examples

``` r
temporal_until(plain_date(2006, 8, 24), plain_date(2019, 1, 31))
#> <duration[1]>
#> [1] P4543D
temporal_until("2006-08-24", plain_date(2019, 1, 31), largest_unit = "year")
#> <duration[1]>
#> [1] P12Y5M7D
temporal_since(plain_time(19, 39), plain_time(9, 0))
#> <duration[1]>
#> [1] PT10H39M
plain_date(2019, 1, 31) - plain_date(2006, 8, 24)
#> <duration[1]>
#> [1] P4543D
temporal_until(
  plain_date_time("2020-01-01T00:00"), plain_date_time("2020-01-02T13:31"),
  smallest_unit = "hour", rounding_mode = "halfExpand"
)
#> <duration[1]>
#> [1] P1DT14H
```
