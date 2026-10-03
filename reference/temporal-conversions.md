# Convert between Temporal types

Conversions that drop or add information, mirroring Temporal's
`.toPlainDate()`, `.toPlainTime()`, `.toPlainDateTime()`, `.toInstant()`
and `.toZonedDateTime()`:

## Usage

``` r
to_plain_date(x)

to_plain_time(x)

to_plain_date_time(x, time = NULL)

to_instant(x)

to_zoned_date_time(
  x,
  time_zone,
  time = NULL,
  disambiguation = c("compatible", "earlier", "later", "reject")
)
```

## Arguments

- x:

  A Temporal object.

- time:

  A plain time (or string) recycled against `x`, or `NULL` for midnight.

- time_zone:

  Time zone identifiers, recycled against `x`.

- disambiguation:

  How to resolve local times in DST gaps and overlaps, see
  [`zoned_date_time()`](https://pedrobtz.github.io/zudate/reference/zoned_date_time.md).

## Value

A plain date, plain time or plain date-time vector.

## Details

- `to_plain_date()` and `to_plain_time()` take the date or time part of
  a plain date-time or (wall clock of a) zoned date-time.

- `to_plain_date_time()` combines a plain date with a plain time
  (midnight when `time` is `NULL`), or takes the wall clock of a zoned
  date-time.

- `to_instant()` drops the time zone of a zoned date-time.

- `to_zoned_date_time()` places an instant, plain date-time or plain
  date in `time_zone`. For plain types the local time is resolved with
  `disambiguation`; a plain date without `time` becomes the start of
  that day. A zoned date-time is converted to `time_zone` keeping the
  exact time.

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
z <- to_zoned_date_time(dt, "Europe/Paris")
z
#> <zoned_date_time[1]>
#> [1] 1995-12-07T03:24:30+01:00[Europe/Paris]
to_instant(z)
#> <instant[1]>
#> [1] 1995-12-07T02:24:30Z
to_zoned_date_time(instant("2020-01-01T00:00Z"), "Asia/Tokyo")
#> <zoned_date_time[1]>
#> [1] 2020-01-01T09:00:00+09:00[Asia/Tokyo]
to_zoned_date_time(plain_date(2020, 3, 29), "Asia/Beirut") # day starts at 01:00
#> <zoned_date_time[1]>
#> [1] 2020-03-29T01:00:00+03:00[Asia/Beirut]
```
