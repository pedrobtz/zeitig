# Format Temporal objects as strings

[`format()`](https://rdrr.io/r/base/format.html) and
[`as.character()`](https://rdrr.io/r/base/character.html) give
Temporal's [`toString()`](https://rdrr.io/r/base/toString.html) output
(RFC 9557 / ISO 8601). The arguments mirror
[`toString()`](https://rdrr.io/r/base/toString.html)'s options; the
defaults print the shortest exact representation.

## Usage

``` r
# S3 method for class 'zeitig_plain_date'
format(x, ..., calendar_name = "auto")

# S3 method for class 'zeitig_plain_time'
format(
  x,
  ...,
  fractional_second_digits = "auto",
  smallest_unit = NULL,
  rounding_mode = "trunc"
)

# S3 method for class 'zeitig_plain_date_time'
format(
  x,
  ...,
  fractional_second_digits = "auto",
  smallest_unit = NULL,
  rounding_mode = "trunc",
  calendar_name = "auto"
)

# S3 method for class 'zeitig_instant'
format(
  x,
  ...,
  fractional_second_digits = "auto",
  smallest_unit = NULL,
  rounding_mode = "trunc",
  time_zone = NULL
)

# S3 method for class 'zeitig_zoned_date_time'
format(
  x,
  ...,
  fractional_second_digits = "auto",
  smallest_unit = NULL,
  rounding_mode = "trunc",
  offset = "auto",
  time_zone_name = "auto",
  calendar_name = "auto"
)
```

## Arguments

- x:

  A Temporal object.

- ...:

  Not used; for compatibility with the generic.

- fractional_second_digits, smallest_unit, rounding_mode:

  See Details.

- offset, time_zone_name, calendar_name, time_zone:

  See Details. Each only applies to the types that have the
  corresponding part.

## Value

A character vector, `NA` for missing elements.

## Details

- `fractional_second_digits`: `"auto"` (default; as many digits as
  needed) or a number from 0 to 9. The value is rounded with
  `rounding_mode` first.

- `smallest_unit`: `"minute"`, `"second"`, `"millisecond"`,
  `"microsecond"` or `"nanosecond"`; overrides
  `fractional_second_digits`. `"minute"` omits the seconds.

- `rounding_mode`: how to round to the requested precision (default
  `"trunc"`).

- `offset` (zoned): `"auto"` prints the UTC offset, `"never"` omits it.

- `time_zone_name` (zoned): `"auto"` prints the `[Zone]` annotation,
  `"never"` omits it and `"critical"` prints `[!Zone]`.

- `calendar_name` (dates and date-times): `"auto"`/`"never"` omit the
  calendar (always ISO 8601), `"always"` prints `[u-ca=iso8601]` and
  `"critical"` `[!u-ca=iso8601]`.

- `time_zone` (instants): print the wall-clock time and offset in this
  zone instead of UTC.

## Examples

``` r
x <- zoned_date_time("2020-01-01T15:23:30.123456789+01:00[Europe/Paris]")
format(x)
#> [1] "2020-01-01T15:23:30.123456789+01:00[Europe/Paris]"
format(x, smallest_unit = "minute")
#> [1] "2020-01-01T15:23+01:00[Europe/Paris]"
format(x, fractional_second_digits = 2, rounding_mode = "halfExpand")
#> [1] "2020-01-01T15:23:30.12+01:00[Europe/Paris]"
format(x, offset = "never", time_zone_name = "critical", calendar_name = "always")
#> [1] "2020-01-01T15:23:30.123456789[!Europe/Paris][u-ca=iso8601]"
format(instant("2020-01-01T00:00Z"), time_zone = "Asia/Tokyo")
#> [1] "2020-01-01T09:00:00+09:00"
format(plain_time("12:00:00.5"), fractional_second_digits = 3)
#> [1] "12:00:00.500"
```
