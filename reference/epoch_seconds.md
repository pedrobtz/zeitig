# Epoch time

The time since 1970-01-01T00:00Z of an instant or zoned date-time.
`epoch_seconds()` and `epoch_milliseconds()` round towards negative
infinity, as Temporal's `epochMilliseconds`; `epoch_nanoseconds()` is
exact and returned as a character vector because R has no 64-bit
integers.

## Usage

``` r
epoch_seconds(x)

epoch_milliseconds(x)

epoch_nanoseconds(x)
```

## Arguments

- x:

  An instant or zoned date-time.

## Value

A double vector (`epoch_seconds()`, `epoch_milliseconds()`) or a
character vector (`epoch_nanoseconds()`).

## See also

Other instant:
[`instant()`](https://pedrobtz.github.io/zudate/reference/instant.md)

## Examples

``` r
x <- instant("2019-03-30T00:45:00.123456789Z")
epoch_seconds(x)
#> [1] 1553906700
epoch_milliseconds(x)
#> [1] 1.553907e+12
epoch_nanoseconds(x)
#> [1] "1553906700123456789"
```
