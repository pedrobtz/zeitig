# Plain times

A `zeitig_plain_time` is a wall-clock time of day with nanosecond
precision and no date or time zone, the equivalent of
[`Temporal.PlainTime`](https://tc39.es/proposal-temporal/docs/plaintime.html).

## Usage

``` r
plain_time(
  hour = 0L,
  minute = 0L,
  second = 0L,
  millisecond = 0L,
  microsecond = 0L,
  nanosecond = 0L,
  ...,
  overflow = c("constrain", "reject")
)

is_plain_time(x)
```

## Arguments

- hour, minute, second, millisecond, microsecond, nanosecond:

  Integer-valued numbers. Fractions are truncated towards zero. `hour`
  may instead be a character vector to parse.

- ...:

  These dots are for future extensions and must be empty.

- overflow:

  How to handle out-of-range values: `"constrain"` (the default) clamps
  every component to its range (e.g. `hour = 25` becomes 23), `"reject"`
  raises an error.

- x:

  An object to test.

## Value

A `zeitig_plain_time` vector.

## Details

`plain_time()` builds times from their components (recycled to a common
length; all default to zero) or, when given a single character vector,
parses ISO 8601 / RFC 9557 strings such as `"15:23:30.123"` or
`"2019-11-18T15:23:30"` (the date part is ignored).

## Examples

``` r
plain_time(19, 39, 9, 68, 346, 205)
#> <plain_time[1]>
#> [1] 19:39:09.068346205
plain_time(c("03:24:30", "15:23:30.5", NA))
#> <plain_time[3]>
#> [1] 03:24:30   15:23:30.5 <NA>      
plain_time(hour = 25, minute = 61) # constrained to 23:59:00
#> <plain_time[1]>
#> [1] 23:59:00
```
