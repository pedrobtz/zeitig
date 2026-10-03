# Plain date-times

A `zeitig_plain_date_time` is a calendar date and a wall-clock time with
nanosecond precision but no time zone, the equivalent of
[`Temporal.PlainDateTime`](https://tc39.es/proposal-temporal/docs/plaindatetime.html).

## Usage

``` r
plain_date_time(
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
  overflow = c("constrain", "reject")
)

is_plain_date_time(x)
```

## Arguments

- year, month, day:

  Integer-valued numbers. `year` may instead be a character vector to
  parse.

- hour, minute, second, millisecond, microsecond, nanosecond:

  Integer-valued numbers. Fractions are truncated towards zero. `hour`
  may instead be a character vector to parse.

- ...:

  These dots are for future extensions and must be empty.

- overflow:

  How to handle out-of-range values: `"constrain"` (the default) clamps
  each component to its range, `"reject"` raises an error.

- x:

  An object to test.

## Value

A `zeitig_plain_date_time` vector.

## Details

`plain_date_time()` builds date-times from their components (recycled to
a common length; the time components default to zero) or, when given a
single character vector, parses RFC 9557 / ISO 8601 strings. A string
with only a date is midnight of that day; offsets and time zone
annotations are ignored, but a UTC designator (`Z`) is an error, as in
Temporal.

## Examples

``` r
plain_date_time(1995, 12, 7, 3, 24, 30, 0, 3, 500)
#> <plain_date_time[1]>
#> [1] 1995-12-07T03:24:30.0000035
plain_date_time(c("1995-12-07T03:24:30.0000035", "2020-01-01"))
#> <plain_date_time[2]>
#> [1] 1995-12-07T03:24:30.0000035 2020-01-01T00:00:00        
```
