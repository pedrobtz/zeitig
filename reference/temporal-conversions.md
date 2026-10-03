# Convert between Temporal types

Conversions that drop or add information, mirroring Temporal's
`.toPlainDate()`, `.toPlainTime()` and `.toPlainDateTime()`:

## Usage

``` r
to_plain_date(x)

to_plain_time(x)

to_plain_date_time(x, time = NULL)
```

## Arguments

- x:

  A Temporal object.

- time:

  A plain time (or string) recycled against `x`, or `NULL` for midnight.

## Value

A plain date, plain time or plain date-time vector.

## Details

- `to_plain_date()` and `to_plain_time()` take the date or time part of
  a plain date-time.

- `to_plain_date_time()` combines a plain date with a plain time
  (midnight when `time` is `NULL`).

## Examples

``` r
dt <- plain_date_time("1995-12-07T03:24:30")
to_plain_date(dt)
#> <plain_date[1]>
#> [1] 1995-12-07
to_plain_time(dt)
#> <plain_time[1]>
#> [1] 03:24:30
to_plain_date_time(plain_date(2006, 8, 24), plain_time(15, 30))
#> <plain_date_time[1]>
#> [1] 2006-08-24T15:30:00
```
