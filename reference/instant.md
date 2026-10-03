# Instants

A `zudate_instant` is an exact point in time with nanosecond precision
and no time zone or calendar, the equivalent of
[`Temporal.Instant`](https://tc39.es/proposal-temporal/docs/instant.html)
(and of `jiff::Timestamp`). The supported range is years -9999 to 9999.

## Usage

``` r
instant(x)

instant_from_epoch(seconds = NULL, milliseconds = NULL, nanoseconds = NULL)

is_instant(x)
```

## Arguments

- x:

  A character vector to parse, or an object to test.

- seconds, milliseconds, nanoseconds:

  Exactly one of these: numbers of units since the epoch, or for
  `nanoseconds` also a character vector of integers. Fractional seconds
  are rounded to the microsecond; fractional milliseconds and
  nanoseconds are truncated.

## Value

A `zudate_instant` vector.

## Details

`instant()` parses RFC 9557 strings, which must include a UTC offset or
`Z` (a time zone annotation alone is not enough). `instant_from_epoch()`
builds instants from a count of seconds, milliseconds or nanoseconds
since 1970-01-01T00:00Z; nanoseconds may be given as strings because
doubles cannot hold present-day epoch nanoseconds exactly.

## See also

Other instant:
[`epoch_seconds()`](https://pedrobtz.github.io/zudate/reference/epoch_seconds.md)

## Examples

``` r
instant(c("1969-07-20T20:17Z", "2020-01-01T12:00:00.5+01:00", NA))
#> <instant[3]>
#> [1] 1969-07-20T20:17:00Z   2020-01-01T11:00:00.5Z <NA>                  
instant_from_epoch(seconds = 0)
#> <instant[1]>
#> [1] 1970-01-01T00:00:00Z
instant_from_epoch(milliseconds = 1553906700000)
#> <instant[1]>
#> [1] 2019-03-30T00:45:00Z
instant_from_epoch(nanoseconds = "1553906700000000001")
#> <instant[1]>
#> [1] 2019-03-30T00:45:00.000000001Z
```
