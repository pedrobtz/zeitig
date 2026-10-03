# Introduction to zudate

zudate brings the [TC39
Temporal](https://tc39.es/proposal-temporal/docs/) date and time model
to R. Every Temporal type is a vectorised R class, and all date/time
logic is delegated to the Rust crate [jiff](https://docs.rs/jiff), which
implements Temporal’s semantics.

``` r

library(zudate)
#> 
#> Attaching package: 'zudate'
#> The following object is masked from 'package:stats':
#> 
#>     offset
```

## The types

| Temporal | zudate | What it represents |
|----|----|----|
| `Temporal.PlainDate` | [`plain_date()`](https://pedrobtz.github.io/zudate/reference/plain_date.md) | a calendar date, no time or time zone |
| `Temporal.PlainTime` | [`plain_time()`](https://pedrobtz.github.io/zudate/reference/plain_time.md) | a wall-clock time, no date or time zone |
| `Temporal.PlainDateTime` | [`plain_date_time()`](https://pedrobtz.github.io/zudate/reference/plain_date_time.md) | a date and a wall-clock time, no time zone |
| `Temporal.Instant` | [`instant()`](https://pedrobtz.github.io/zudate/reference/instant.md) | an exact point in time (nanosecond precision) |
| `Temporal.ZonedDateTime` | [`zoned_date_time()`](https://pedrobtz.github.io/zudate/reference/zoned_date_time.md) | an exact time plus the time zone used to view it |
| `Temporal.Duration` | [`duration()`](https://pedrobtz.github.io/zudate/reference/duration.md) | a length of time in calendar and clock units |
| `Temporal.Now` | [`now_instant()`](https://pedrobtz.github.io/zudate/reference/now_instant.md), … | the current time |

Each constructor accepts components or [RFC
9557](https://www.rfc-editor.org/rfc/rfc9557) strings:

``` r

plain_date(2006, 8, 24)
#> <plain_date[1]>
#> [1] 2006-08-24
plain_time(c("03:24:30", "15:23:30.123456789"))
#> <plain_time[2]>
#> [1] 03:24:30           15:23:30.123456789
plain_date_time("1995-12-07T03:24:30")
#> <plain_date_time[1]>
#> [1] 1995-12-07T03:24:30
instant("1969-07-20T20:17Z")
#> <instant[1]>
#> [1] 1969-07-20T20:17:00Z
zoned_date_time("1995-12-07T03:24:30-08:00[America/Los_Angeles]")
#> <zoned_date_time[1]>
#> [1] 1995-12-07T03:24:30-08:00[America/Los_Angeles]
duration(hours = 1, minutes = 30)
#> <duration[1]>
#> [1] PT1H30M
```

They are ordinary vectors: they recycle, hold `NA`, live in data frames
and survive [`saveRDS()`](https://rdrr.io/r/base/readRDS.html).

``` r

d <- plain_date(2024, 1:4, 31)
d
#> <plain_date[4]>
#> [1] 2024-01-31 2024-02-29 2024-03-31 2024-04-30
data.frame(date = d, weekday = day_of_week(d))
#>         date weekday
#> 1 2024-01-31       3
#> 2 2024-02-29       4
#> 3 2024-03-31       7
#> 4 2024-04-30       2
sort(c(d, NA), na.last = TRUE)
#> <plain_date[5]>
#> [1] 2024-01-31 2024-02-29 2024-03-31 2024-04-30 <NA>
```

Out-of-range components are constrained by default, as in Temporal; use
`overflow = "reject"` to get an error instead.

``` r

plain_date(2021, 2, 31)
#> <plain_date[1]>
#> [1] 2021-02-28
plain_date(2021, 2, 31, overflow = "reject")
#> Error in `plain_date()`:
#> ! parameter 'day' with value 31 is not in the required range of 1..=28 (element 1)
```

## Fields

``` r

x <- plain_date_time("2021-01-03T15:23:30.123456789")
year(x)
#> [1] 2021
week_of_year(x)
#> [1] 53
year_of_week(x)
#> [1] 2020
c(millisecond(x), microsecond(x), nanosecond(x))
#> [1] 123 456 789
temporal_fields(x)
#>   year month day hour minute second millisecond microsecond nanosecond
#> 1 2021     1   3   15     23     30         123         456        789
```

## Arithmetic

Durations are added with `+`/`-` or
[`temporal_add()`](https://pedrobtz.github.io/zudate/reference/temporal_add.md).
Adding months clamps the day of month:

``` r

plain_date(2021, 1, 31) + duration(months = 1)
#> <plain_date[1]>
#> [1] 2021-02-28
plain_time(23, 30) + duration(hours = 1)
#> <plain_time[1]>
#> [1] 00:30:00
```

[`temporal_until()`](https://pedrobtz.github.io/zudate/reference/temporal_until.md)
and
[`temporal_since()`](https://pedrobtz.github.io/zudate/reference/temporal_until.md)
compute differences, balanced up to `largest_unit` and rounded to
`smallest_unit`:

``` r

a <- plain_date(2006, 8, 24)
b <- plain_date(2019, 1, 31)
temporal_until(a, b)
#> <duration[1]>
#> [1] P4543D
temporal_until(a, b, largest_unit = "year")
#> <duration[1]>
#> [1] P12Y5M7D
b - a
#> <duration[1]>
#> [1] P4543D
```

Rounding uses Temporal’s rounding modes:

``` r

t <- plain_time(19, 39, 9, 68, 346, 205)
temporal_round(t, "hour")
#> <plain_time[1]>
#> [1] 20:00:00
temporal_round(t, "minute", rounding_increment = 15, rounding_mode = "floor")
#> <plain_time[1]>
#> [1] 19:30:00
temporal_round(duration(minutes = 130), largest_unit = "hour")
#> <duration[1]>
#> [1] PT2H10M
duration_total(duration(months = 1), "day", relative_to = plain_date(2020, 2, 1))
#> [1] 29
```

## Formatting and parsing

[`format()`](https://rdrr.io/r/base/format.html) follows Temporal’s
[`toString()`](https://rdrr.io/r/base/toString.html) options, and
[`temporal_strftime()`](https://pedrobtz.github.io/zudate/reference/temporal_strftime.md)
/
[`temporal_strptime()`](https://pedrobtz.github.io/zudate/reference/temporal_strftime.md)
use strftime-style patterns:

``` r

z <- zoned_date_time("2020-01-01T15:23:30.123456789+01:00[Europe/Paris]")
format(z, smallest_unit = "minute")
#> [1] "2020-01-01T15:23+01:00[Europe/Paris]"
format(z, fractional_second_digits = 3, time_zone_name = "never")
#> [1] "2020-01-01T15:23:30.123+01:00"
temporal_strftime(z, "%A %d %B %Y, %H:%M %Z")
#> [1] "Wednesday 01 January 2020, 15:23 CET"
temporal_strptime("15/07/2024", "%d/%m/%Y", "plain_date")
#> <plain_date[1]>
#> [1] 2024-07-15
```

## Base R interoperability

``` r

as_plain_date(Sys.Date())
#> <plain_date[1]>
#> [1] 2026-10-03
as.Date(plain_date(2020, 2, 29))
#> [1] "2020-02-29"
as_zoned_date_time(as.POSIXct("2020-07-01 12:00:00", tz = "Europe/Paris"))
#> <zoned_date_time[1]>
#> [1] 2020-07-01T12:00:00+02:00[Europe/Paris]
as.POSIXct(instant("2020-01-01T00:00:00.25Z"))
#> [1] "2020-01-01 00:00:00 UTC"
as_duration(as.difftime(90, units = "mins"))
#> <duration[1]>
#> [1] PT90M
```

`POSIXct` holds seconds in a double, so conversions to and from it are
rounded to the microsecond. Use the Temporal types (or
[`epoch_nanoseconds()`](https://pedrobtz.github.io/zudate/reference/epoch_seconds.md))
when you need nanoseconds.

## Differences from Temporal

zudate supports only the ISO 8601 calendar, and a few behaviours follow
jiff or R conventions; they are listed in the package’s design notes.
The most visible ones:

- `x - y` for two values of the same type is `temporal_since(x, y)`.
- `==` and `<` on zoned date-times compare the exact time only;
  [`temporal_equals()`](https://pedrobtz.github.io/zudate/reference/temporal_compare.md)
  also compares the time zone.
- `==` on durations compares fields (`PT1H` is not `==` to `PT60M`); use
  [`duration_compare()`](https://pedrobtz.github.io/zudate/reference/duration_total.md).
