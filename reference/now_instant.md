# The current time

The equivalent of `Temporal.Now`: the current exact time as an instant,
or as a zoned date-time or plain value in a time zone.

## Usage

``` r
now_instant()

now_zoned_date_time(time_zone = NULL)

now_plain_date_time(time_zone = NULL)

now_plain_date(time_zone = NULL)

now_plain_time(time_zone = NULL)

now_time_zone()
```

## Arguments

- time_zone:

  A time zone identifier, or `NULL` for the session time zone.

## Value

A length-one vector of the requested type (`now_time_zone()`: a string).

## Details

The default time zone is the session's: the `TZ` environment variable
when it names a valid zone, otherwise
[`Sys.timezone()`](https://rdrr.io/r/base/timezones.html), otherwise
`"UTC"`. `now_time_zone()` returns that identifier. The same default
applies to `POSIXct` values without a `tzone` attribute.

## Examples

``` r
now_instant()
#> <instant[1]>
#> [1] 2026-10-04T07:05:36.956932776Z
now_zoned_date_time("Asia/Tokyo")
#> <zoned_date_time[1]>
#> [1] 2026-10-04T16:05:36.957775585+09:00[Asia/Tokyo]
now_plain_date()
#> <plain_date[1]>
#> [1] 2026-10-04
now_plain_time("UTC")
#> <plain_time[1]>
#> [1] 07:05:36.963674407
now_plain_date_time()
#> <plain_date_time[1]>
#> [1] 2026-10-04T07:05:36.964748605
now_time_zone()
#> [1] "UTC"
```
