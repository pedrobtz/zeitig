# Compare Temporal objects

Temporal objects support the usual comparison operators (`<`, `==`,
...), [`sort()`](https://rdrr.io/r/base/sort.html),
[`order()`](https://rdrr.io/r/base/order.html),
[`min()`](https://rdrr.io/r/base/Extremes.html)/[`max()`](https://rdrr.io/r/base/Extremes.html)
and [`unique()`](https://rdrr.io/r/base/unique.html), all with
Temporal's `compare()` ordering. `temporal_compare()` is the equivalent
of `Temporal.X.compare(a, b)` and `temporal_equals()` of `a.equals(b)`.

## Usage

``` r
temporal_compare(x, y)

temporal_equals(x, y)
```

## Arguments

- x, y:

  Temporal objects of the same type (or strings), recycled to a common
  length.

## Value

`temporal_compare()`: an integer vector of -1, 0 and 1.
`temporal_equals()`: a logical vector. Both are `NA` where either input
is missing.

## Details

Character vectors are parsed as the type of the other argument.

## Examples

``` r
a <- plain_date(c("2020-01-01", "2021-06-30"))
temporal_compare(a, "2021-01-01")
#> [1] -1  1
temporal_equals(a, "2020-01-01")
#> [1]  TRUE FALSE
sort(plain_time(c("12:00", "08:30", "23:59:59.5")))
#> <plain_time[3]>
#> [1] 08:30:00   12:00:00   23:59:59.5
```
